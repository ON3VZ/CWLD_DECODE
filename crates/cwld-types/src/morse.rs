//! Morse patterns and timing (PARIS standard).
//!
//! One unit lasts 1.2 / WPM seconds. A dit is 1 unit, a dah 3, the gap inside a
//! character 1, between characters 3 and between words 7.

/// The dit/dah pattern of a character, if CWLD can send it.
pub fn pattern(c: char) -> Option<&'static str> {
    Some(match c.to_ascii_uppercase() {
        'A' => ".-",
        'B' => "-...",
        'C' => "-.-.",
        'D' => "-..",
        'E' => ".",
        'F' => "..-.",
        'G' => "--.",
        'H' => "....",
        'I' => "..",
        'J' => ".---",
        'K' => "-.-",
        'L' => ".-..",
        'M' => "--",
        'N' => "-.",
        'O' => "---",
        'P' => ".--.",
        'Q' => "--.-",
        'R' => ".-.",
        'S' => "...",
        'T' => "-",
        'U' => "..-",
        'V' => "...-",
        'W' => ".--",
        'X' => "-..-",
        'Y' => "-.--",
        'Z' => "--..",
        '0' => "-----",
        '1' => ".----",
        '2' => "..---",
        '3' => "...--",
        '4' => "....-",
        '5' => ".....",
        '6' => "-....",
        '7' => "--...",
        '8' => "---..",
        '9' => "----.",
        '.' => ".-.-.-",
        ',' => "--..--",
        '?' => "..--..",
        '/' => "-..-.",
        '=' => "-...-",
        '+' => ".-.-.",
        '-' => "-....-",
        ':' => "---...",
        '\'' => ".----.",
        '"' => ".-..-.",
        '(' => "-.--.",
        ')' => "-.--.-",
        '@' => ".--.-.",
        _ => return None,
    })
}

fn char_units(pattern: &str) -> u32 {
    let elements: u32 = pattern.chars().map(|e| if e == '-' { 3 } else { 1 }).sum();
    let gaps = pattern.len().saturating_sub(1) as u32;
    elements + gaps
}

/// Total length of a text in units, without a trailing word gap.
/// Returns `None` if the text contains a character that cannot be sent.
pub fn duration_units(text: &str) -> Option<u32> {
    let mut total = 0u32;
    let mut seen_char = false;
    let mut word_gap = false;
    for c in text.chars() {
        if c == ' ' {
            if seen_char {
                word_gap = true;
            }
            continue;
        }
        let p = pattern(c)?;
        if seen_char {
            total += if word_gap { 7 } else { 3 };
        }
        word_gap = false;
        total += char_units(p);
        seen_char = true;
    }
    Some(total)
}

/// Duration in seconds at the given speed.
pub fn duration_seconds(text: &str, wpm: u8) -> Option<f64> {
    if wpm == 0 {
        return None;
    }
    duration_units(text).map(|u| f64::from(u) * 1.2 / f64::from(wpm))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paris_is_the_standard_word() {
        // "PARIS" is 43 units, plus the 7 unit word gap after it makes the famous 50.
        assert_eq!(duration_units("PARIS"), Some(43));
        assert_eq!(duration_units("PARIS PARIS"), Some(43 + 7 + 43));
    }

    #[test]
    fn spaces_are_collapsed_and_trimmed() {
        assert_eq!(duration_units("  E   E  "), Some(1 + 7 + 1));
        assert_eq!(duration_units(""), Some(0));
    }

    #[test]
    fn unknown_characters_are_rejected() {
        assert_eq!(duration_units("CQ #"), None);
        assert_eq!(duration_units("é"), None);
    }

    #[test]
    fn seconds_follow_the_paris_rule() {
        // At 20 WPM one unit is 60 ms, so PARIS (43 units) takes 2.58 s.
        let s = duration_seconds("PARIS", 20).unwrap();
        assert!((s - 2.58).abs() < 1e-9);
        assert_eq!(duration_seconds("PARIS", 0), None);
    }

    #[test]
    fn lowercase_works() {
        assert_eq!(duration_units("cq"), duration_units("CQ"));
    }
}
