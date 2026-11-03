//! Preparing text for a radio: cleaning, prosigns, cutting into blocks, timing.
//!
//! Pure functions, no radio needed. Both drivers use them.

use cwld_types::morse;

/// How a prosign must be written for a given radio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProsignStyle {
    /// Icom: `^AR`. **[TEST]** on the IC-7300 MK II.
    Caret,
    /// QRP Labs QMX in TS-480 compatibility mode: one special character. **[TEST]**
    Qmx,
}

/// Prosign names that CWLD knows, with the QMX character and the Morse pattern.
const PROSIGNS: [(&str, char, &str); 8] = [
    ("AR", '_', ".-.-."),
    ("AS", '<', ".-..."),
    ("BT", '[', "-...-"),
    ("HH", '#', "........"),
    ("SK", '>', "...-.-"),
    ("KN", '=', "-.--."),
    ("BK", '\\', "-...-.-"),
    ("SN", '%', "...-."),
];

fn prosign_for(name: &str, style: ProsignStyle) -> Option<String> {
    PROSIGNS.iter().find(|(n, _, _)| *n == name).map(|(n, c, _)| match style {
        ProsignStyle::Caret => format!("^{n}"),
        ProsignStyle::Qmx => c.to_string(),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prepared {
    /// Words, ready for the radio (prosigns already translated).
    pub words: Vec<String>,
    /// Characters that could not be sent and were left out.
    pub dropped: Vec<char>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyerError {
    Empty,
    /// A single word that does not fit in one block.
    WordTooLong { word: String, max: usize },
}

fn plain_ok(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | ',' | '?' | '/' | '=' | '+' | '-' | '@')
}

/// Cleans text: capitals, one space between words, unsupported characters out,
/// `<AR>` style prosigns translated for the radio.
pub fn prepare(text: &str, style: ProsignStyle) -> Result<Prepared, KeyerError> {
    let mut words = Vec::new();
    let mut dropped = Vec::new();
    for raw in text.split_whitespace() {
        let up = raw.to_ascii_uppercase();
        let mut word = String::new();
        let mut chars = up.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '<' {
                let mut name = String::new();
                let mut closed = false;
                for n in chars.by_ref() {
                    if n == '>' {
                        closed = true;
                        break;
                    }
                    name.push(n);
                }
                match (closed, prosign_for(&name, style)) {
                    (true, Some(p)) => word.push_str(&p),
                    _ => dropped.extend(format!("<{name}{}", if closed { ">" } else { "" }).chars()),
                }
            } else if plain_ok(c) {
                // In QMX compatibility mode `=` means KN. A plain `=` (BT) is sent as the BT prosign.
                if c == '=' && style == ProsignStyle::Qmx {
                    word.push('[');
                } else {
                    word.push(c);
                }
            } else {
                dropped.push(c);
            }
        }
        if !word.is_empty() {
            words.push(word);
        }
    }
    if words.is_empty() {
        return Err(KeyerError::Empty);
    }
    Ok(Prepared { words, dropped })
}

/// Cuts words into blocks of at most `max` characters, on word boundaries only.
/// Every block except the last ends with a space, so the word gap survives the cut.
pub fn chunk_words(words: &[String], max: usize) -> Result<Vec<String>, KeyerError> {
    let mut chunks: Vec<String> = Vec::new();
    let mut cur = String::new();
    for w in words {
        let len = w.chars().count();
        if len > max {
            return Err(KeyerError::WordTooLong { word: w.clone(), max });
        }
        // A block needs room for the word and, unless it is the last block, a trailing space.
        let needed = if cur.is_empty() { len } else { cur.chars().count() + 1 + len };
        if needed + 1 <= max || (cur.is_empty() && len <= max) {
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(w);
        } else {
            cur.push(' ');
            chunks.push(std::mem::take(&mut cur));
            cur.push_str(w);
        }
    }
    if !cur.is_empty() {
        chunks.push(cur);
    }
    Ok(chunks)
}

/// Length in Morse units of a block as it is sent to the radio (`^AR` and QMX prosign
/// characters are sent as one character without gaps between the letters).
pub fn units(block: &str, style: ProsignStyle) -> u32 {
    let mut total = 0u32;
    let mut first = true;
    let mut gap_before_word = false;
    let mut chars = block.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ' ' {
            if !first {
                gap_before_word = true;
            }
            continue;
        }
        let pat: Option<String> = if style == ProsignStyle::Caret && c == '^' {
            let name: String = chars.by_ref().take(2).collect();
            PROSIGNS.iter().find(|(n, _, _)| *n == name).map(|(_, _, p)| p.to_string())
        } else if style == ProsignStyle::Qmx {
            PROSIGNS.iter().find(|(_, ch, _)| *ch == c).map(|(_, _, p)| p.to_string()).or_else(|| morse::pattern(c).map(|p| p.to_string()))
        } else {
            morse::pattern(c).map(|p| p.to_string())
        };
        let Some(p) = pat else { continue };
        if !first {
            total += if gap_before_word { 7 } else { 3 };
        }
        gap_before_word = false;
        let elements: u32 = p.chars().map(|e| if e == '-' { 3 } else { 1 }).sum();
        total += elements + p.len().saturating_sub(1) as u32;
        first = false;
    }
    total
}

/// Longest message the driver accepts, in seconds. A guard against a pasted wall of text.
pub const MAX_MESSAGE_SECONDS: u64 = 60;

/// Estimated sending time of a block in milliseconds.
pub fn estimate_ms(block: &str, wpm: u8, style: ProsignStyle) -> u64 {
    let unit_ms = 1200.0 / f64::from(wpm.max(1));
    (f64::from(units(block, style)) * unit_ms).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> Vec<String> {
        s.split(' ').map(String::from).collect()
    }

    #[test]
    fn cleans_and_uppercases() {
        let p = prepare("  cq   cq de on3vz  ", ProsignStyle::Caret).unwrap();
        assert_eq!(p.words, w("CQ CQ DE ON3VZ"));
        assert!(p.dropped.is_empty());
    }

    #[test]
    fn drops_what_cannot_be_sent_and_reports_it() {
        let p = prepare("CQ! DE ON3VZ#", ProsignStyle::Caret).unwrap();
        assert_eq!(p.words, w("CQ DE ON3VZ"));
        assert_eq!(p.dropped, vec!['!', '#']);
    }

    #[test]
    fn empty_text_is_an_error() {
        assert_eq!(prepare("  \t ", ProsignStyle::Caret), Err(KeyerError::Empty));
        assert_eq!(prepare("!!!", ProsignStyle::Caret), Err(KeyerError::Empty));
    }

    #[test]
    fn prosigns_per_radio() {
        assert_eq!(prepare("73 <SK>", ProsignStyle::Caret).unwrap().words, w("73 ^SK"));
        assert_eq!(prepare("73 <SK>", ProsignStyle::Qmx).unwrap().words, w("73 >"));
        assert_eq!(prepare("<AR>", ProsignStyle::Qmx).unwrap().words, w("_"));
    }

    #[test]
    fn unknown_prosign_is_dropped_not_sent() {
        let p = prepare("K <XX> K", ProsignStyle::Caret).unwrap();
        assert_eq!(p.words, w("K K"));
        assert!(!p.dropped.is_empty());
    }

    #[test]
    fn chunks_never_exceed_the_limit_and_never_cut_words() {
        let text = "CQ CQ CQ DE ON3VZ ON3VZ ON3VZ PSE K CQ CQ CQ DE ON3VZ ON3VZ ON3VZ PSE K";
        let chunks = chunk_words(&w(text), 30).unwrap();
        assert!(chunks.len() >= 2);
        for c in &chunks {
            assert!(c.chars().count() <= 30, "{c:?}");
        }
        // Joining the blocks gives the original text back, word gaps included.
        assert_eq!(chunks.concat(), text);
        // Only the last block lacks the trailing space.
        for c in &chunks[..chunks.len() - 1] {
            assert!(c.ends_with(' '));
        }
        assert!(!chunks.last().unwrap().ends_with(' '));
    }

    #[test]
    fn short_text_is_one_block_without_trailing_space() {
        assert_eq!(chunk_words(&w("CQ DE ON3VZ"), 30).unwrap(), vec!["CQ DE ON3VZ".to_string()]);
    }

    #[test]
    fn word_longer_than_a_block_is_refused() {
        let long = "A".repeat(31);
        assert!(matches!(chunk_words(&[long], 30), Err(KeyerError::WordTooLong { max: 30, .. })));
    }

    #[test]
    fn word_of_exactly_the_limit_fits() {
        let exact = "A".repeat(30);
        assert_eq!(chunk_words(&[exact.clone()], 30).unwrap(), vec![exact]);
    }

    #[test]
    fn units_follow_paris() {
        // PARIS plus the word gap is 50 units.
        assert_eq!(units("PARIS", ProsignStyle::Caret) + 7, 50);
        assert_eq!(units("PARIS", ProsignStyle::Qmx) + 7, 50);
    }

    #[test]
    fn prosign_is_one_character_without_inner_gaps() {
        // AR as a prosign is ".-.-." = 1+1+3+1+1+3+1+1+1 = 13 units.
        assert_eq!(units("^AR", ProsignStyle::Caret), 13);
        assert_eq!(units("_", ProsignStyle::Qmx), 13);
    }

    #[test]
    fn estimate_at_20_wpm() {
        // 50 units at 20 WPM is 3 seconds.
        assert_eq!(estimate_ms("PARIS PARIS", 20, ProsignStyle::Caret), 6_000 - 420);
    }
}
