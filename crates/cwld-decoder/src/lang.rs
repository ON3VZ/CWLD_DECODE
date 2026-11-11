//! Word knowledge: a small model of what CW operators actually send.
//!
//! The decoder gives per character the best reading and its alternatives. At the end of every
//! word the corrector tries the combinations of the uncertain characters and picks the one
//! that is most likely as a whole: the character probabilities times a bonus for words that
//! are common in CW (Q-codes, abbreviations, QSO words), for callsign shapes and for numbers.
//! A sure character is never changed, and only same-length readings are considered.
//!
//! It also removes "junk words": one or two characters made of a single short mark each
//! (E, T, I, M, N, A) that barely stood out of the noise. Noise blips decode exactly like that.

use crate::decoder::{DecodeUpdate, TentativeChar};
use std::collections::HashSet;

/// Common CW vocabulary. Generic on purpose: no names or places from the test bench.
const WORDS: &[&str] = &[
    // Q-codes
    "QRA", "QRG", "QRH", "QRI", "QRK", "QRL", "QRM", "QRN", "QRO", "QRP", "QRQ", "QRS", "QRT", "QRU", "QRV", "QRX", "QRZ",
    "QSA", "QSB", "QSD", "QSK", "QSL", "QSO", "QSP", "QST", "QSX", "QSY", "QTC", "QTH", "QTR",
    // Abbreviations and procedure
    "CQ", "DE", "K", "KN", "BK", "SK", "AR", "AS", "BT", "R", "RR", "TU", "TNX", "TKS", "THX", "FB", "OM", "YL", "XYL",
    "HR", "HW", "HI", "UR", "URS", "RST", "NAME", "OP", "OPR", "RIG", "ANT", "PWR", "WX", "TEMP", "ES", "GL", "GM", "GA",
    "GE", "GN", "GD", "DX", "AGN", "PSE", "CPY", "CPI", "CUL", "CU", "SRI", "FER", "FR", "ABT", "WID", "WKD", "WKG",
    "HPE", "BURO", "VIA", "MNI", "OK", "NR", "NIL", "INFO", "EQUIP", "TX", "RX", "TRX", "CFM", "CLG", "CALL", "CL",
    "DR", "ENUF", "FM", "GUD", "HV", "LID", "N", "NW", "OB", "OT", "PSED", "RPT", "SIGS", "SIG", "SN", "SOLID", "STN",
    "TEST", "TMW", "TT", "U", "W", "WRK", "YRS", "AGE", "BEAM", "DIPOLE", "VERT", "VERTICAL", "YAGI", "LOOP", "WIRE",
    "EFHW", "INV", "WATTS", "W", "KW", "QRPP", "POTA", "SOTA", "WWFF", "IOTA", "CONTEST", "PORTABLE", "MOBILE",
    "QRPER", "CW", "SSB", "FT8", "73", "72", "88", "55", "5NN", "599", "579", "559", "569", "589", "449", "339", "TU",
    // Weather and plain words that come up in every QSO
    "SUNNY", "SUN", "CLOUDY", "CLDY", "RAIN", "SNOW", "WIND", "WINDY", "FOG", "COLD", "WARM", "HOT", "FINE", "NICE",
    "AND", "THE", "TO", "FOR", "IN", "ON", "AT", "IS", "ME", "MY", "YOU", "YOUR", "WAS", "ALSO", "HERE", "THERE", "BUT",
    "NOT", "ALL", "GOOD", "VERY", "MANY", "THANKS", "BEST", "LUCK", "SEE", "AGAIN", "SOON", "NEXT", "TIME", "BAND",
    "CONDX", "CONDITIONS", "NOISE", "QRMS", "HOME", "OFFICE", "WORK", "OLD", "MAN", "DEAR", "FRIEND", "EYEBALL",
    // Plain English of a relaxed QSO: the commonest words and the usual subjects.
    "A", "I", "AM", "AN", "AS", "AT", "BE", "BY", "DO", "GO", "HE", "IF", "IN", "IS", "IT", "ME", "MY", "NO", "OF",
    "ON", "OR", "SO", "TO", "UP", "US", "WE", "ALL", "AND", "ANY", "ARE", "BAD", "BIG", "BIT", "BUT", "CAN", "DAY",
    "DID", "END", "FAR", "FEW", "FIX", "FOR", "GET", "GOT", "HAD", "HAS", "HER", "HIM", "HIS", "HOW", "ITS", "LET",
    "LOT", "NEW", "NOT", "NOW", "OFF", "OLD", "ONE", "OUR", "OUT", "OWN", "PUT", "RAN", "RUN", "SAW", "SAY", "SEE",
    "SET", "SHE", "THE", "TOO", "TRY", "TWO", "USE", "WAS", "WAY", "WHO", "WHY", "YES", "YET", "YOU", "ALSO", "AWAY",
    "BACK", "BEEN", "BEST", "BOTH", "BUSY", "CAME", "COME", "DAYS", "DOES", "DONE", "DOWN", "EACH", "EVEN", "EVER",
    "FEEL", "FIND", "FINE", "FROM", "GAVE", "GIVE", "GOES", "GONE", "GOOD", "HAVE", "HEAR", "HELP", "HERE", "HOME",
    "HOPE", "INTO", "JUST", "KEEP", "KNOW", "LAST", "LATE", "LEFT", "LIFE", "LIKE", "LINE", "LONG", "LOOK", "MADE",
    "MAKE", "MANY", "MORE", "MOST", "MUCH", "MUST", "NEAR", "NEED", "NEXT", "NICE", "ONLY", "OPEN", "OVER", "PART",
    "SAID", "SAME", "SOME", "SOON", "SUCH", "SURE", "TAKE", "TELL", "THAN", "THAT", "THEM", "THEN", "THEY", "THIS",
    "TIME", "TOLD", "TOOK", "USED", "VERY", "WANT", "WEEK", "WELL", "WENT", "WERE", "WHAT", "WHEN", "WILL", "WITH",
    "WORK", "YEAR", "YOUR", "ABOUT", "AFTER", "AGAIN", "BEING", "COULD", "DIED", "EARLY", "EVERY", "FIRST", "GOING",
    "GREAT", "HEARD", "HOURS", "LATER", "MIGHT", "MONTH", "NEVER", "NIGHT", "OTHER", "QUITE", "RIGHT", "SINCE",
    "SMALL", "STILL", "THEIR", "THERE", "THESE", "THING", "THINK", "THOSE", "THREE", "TODAY", "UNDER", "UNTIL",
    "WATCH", "WEEKS", "WHERE", "WHICH", "WHILE", "WORLD", "WOULD", "YEARS", "ALWAYS", "AROUND", "BEFORE", "BETTER",
    "CALLED", "CHANGE", "COMING", "DURING", "ENOUGH", "FAILED", "FRIEND", "LITTLE", "PEOPLE", "REALLY", "SHOULD",
    "THANKS", "THINGS", "TRYING", "ANOTHER", "BECAUSE", "BETWEEN", "EVENING", "MORNING", "NOTHING", "THROUGH",
    "TONIGHT", "WEATHER", "WORKING", "WITHOUT", "TOMORROW", "YESTERDAY", "WEEKEND", "HOLIDAY", "GARDEN", "HOUSE",
    "WIFE", "FAMILY", "RADIO", "STATION", "ANTENNA", "AERIAL", "TOWER", "MAST", "CABLE", "COAX", "TUNER", "KEYER",
    "PADDLE", "SIGNAL", "SIGNALS", "STRONG", "WEAK", "COPY", "SORRY", "LOTS", "REPORT", "POWER", "RUNNING", "USING",
    "SCREEN", "LAPTOP", "MEET", "COMPUTER", "CHECKED", "CHECK", "SHOP", "TOWN", "CITY", "MILES", "NORTH", "SOUTH", "EAST",
    "WEST", "RETIRED", "LICENSED", "CLUB", "MEMBER", "CALLING", "CONTACT", "PLEASE", "REPEAT", "SLOW", "FAST",
    "SPEED", "HEAVY", "WET", "DRY", "DEGREES", "FROST", "STORM", "CLEAR", "CLOUD", "WUD", "CUD", "SHUD", "ABT",
    "BCNU", "BTU", "CONDX", "GUD", "HPE", "SEZ", "TMW", "TNX", "VY", "WL", "YR",
];

pub struct WordModel {
    words: HashSet<String>,
    /// Callsigns heard a moment ago, newest last. A station repeats its call.
    recent: std::collections::VecDeque<String>,
    /// Callsigns given by the operator (his own, those in the QSO panel): never forgotten.
    pinned: Vec<String>,
    by_elements: std::collections::HashMap<String, Vec<String>>,
}

impl Default for WordModel {
    fn default() -> Self {
        let mut m = WordModel { words: HashSet::new(), recent: std::collections::VecDeque::new(), pinned: Vec::new(), by_elements: std::collections::HashMap::new() };
        for w in WORDS {
            m.add(w);
        }
        m
    }
}

/// The dots and dashes of a word in one row, without the gaps between the characters.
pub fn elements(w: &str) -> Option<String> {
    let mut out = String::new();
    for c in w.chars() {
        out.push_str(cwld_types::morse::pattern(c)?);
    }
    Some(out)
}

impl WordModel {
    pub fn known(&self, w: &str) -> bool {
        self.words.contains(w)
    }

    /// Adds words (for example the operator's own callsign, name and QTH, or worked calls).
    pub fn add(&mut self, w: &str) {
        let w = w.trim().to_ascii_uppercase();
        if looks_like_callsign(&w) && w.chars().count() >= 4 && !self.pinned.contains(&w) {
            self.pinned.push(w.clone());
        }
        if !w.is_empty() && self.words.insert(w.clone()) {
            if let Some(el) = elements(&w) {
                self.by_elements.entry(el).or_default().push(w);
            }
        }
    }

    /// Remembers a callsign that came through clearly.
    pub fn heard(&mut self, w: &str) {
        if !looks_like_callsign(w) || w.chars().count() < 4 {
            return;
        }
        self.recent.retain(|x| x != w);
        self.recent.push_back(w.to_string());
        while self.recent.len() > 12 {
            self.recent.pop_front();
        }
    }

    pub fn heard_before(&self, w: &str) -> bool {
        self.recent.iter().any(|x| x == w) || self.pinned.iter().any(|x| x == w)
    }

    /// The known words and the callsigns heard before that are made of exactly this row of
    /// dots and dashes (see `elements`). Used to mend a character that a pause cut in two, or
    /// two characters that ran together: the elements are right, only a boundary is wrong.
    pub fn same_elements(&self, el: &str) -> Vec<&str> {
        let mut out: Vec<&str> = self.recent.iter().filter(|c| elements(c).as_deref() == Some(el)).map(|c| c.as_str()).collect();
        if let Some(v) = self.by_elements.get(el) {
            out.extend(v.iter().map(|w| w.as_str()));
        }
        out
    }

    /// Known words of at least two characters whose row of dots and dashes has this length.
    pub fn words_with_elements(&self, n: usize) -> impl Iterator<Item = &str> {
        self.by_elements.iter().filter(move |(el, _)| el.len() == n).flat_map(|(_, v)| v.iter()).filter(|w| w.chars().count() >= 2).map(|w| w.as_str())
    }

    /// The callsigns heard a moment ago, newest first.
    pub fn recent_calls(&self) -> impl Iterator<Item = &str> {
        self.recent.iter().rev().chain(self.pinned.iter()).map(|c| c.as_str())
    }

    /// Log bonus (nats) for a whole word.
    pub fn score(&self, w: &str) -> f32 {
        if self.known(w) {
            3.0
        } else if looks_like_callsign(w) {
            2.0
        } else if !w.is_empty() && w.chars().all(|c| c.is_ascii_digit()) {
            1.0
        } else {
            0.0
        }
    }
}

/// Prefix (1 to 3 characters, at least one letter), one digit, suffix of 1 to 4 letters,
/// optionally "/P", "/M", "/QRP" or a portable prefix. ON3VZ, DL1ABC, F5XYZ, 9A1A, ON3VZ/P.
pub fn looks_like_callsign(w: &str) -> bool {
    let main = w.split('/').max_by_key(|p| p.len()).unwrap_or(w);
    let chars: Vec<char> = main.chars().collect();
    let Some(d) = chars.iter().rposition(|c| c.is_ascii_digit()) else { return false };
    let (prefix, suffix) = (&chars[..d], &chars[d + 1..]);
    (1..=3).contains(&prefix.len())
        && prefix.iter().all(|c| c.is_ascii_alphanumeric())
        && prefix.iter().any(|c| c.is_ascii_alphabetic())
        && (1..=4).contains(&suffix.len())
        && suffix.iter().all(|c| c.is_ascii_alphabetic())
}

#[derive(Clone, Debug)]
struct Pending {
    c: char,
    conf: f32,
    alts: Vec<(char, f32)>,
    strength: f32,
}

pub struct WordCorrector {
    pub model: WordModel,
    pub enabled: bool,
    /// Characters below this mark strength count as possible noise blips.
    pub junk_strength: f32,
    word: Vec<Pending>,
    last: Option<char>,
}

const UNSURE: f32 = 0.9;
const MAX_COMBOS: usize = 128;

impl WordCorrector {
    pub fn new(enabled: bool) -> Self {
        WordCorrector { model: WordModel::default(), enabled, junk_strength: 3.0, word: Vec::new(), last: None }
    }

    /// Takes the decoder's new final text. Returns (text, confidence per character) that is
    /// final now: whole words, each followed by its space.
    pub fn push(&mut self, up: &DecodeUpdate) -> Vec<(char, f32)> {
        let mut out = Vec::new();
        for (i, c) in up.committed.chars().enumerate() {
            let conf = up.committed_conf.get(i).copied().unwrap_or(1.0);
            if c == ' ' {
                out.extend(self.finish_word());
                // A space only after a word, never two in a row (a dropped junk word leaves
                // nothing to separate). Words and their space can arrive in different calls.
                let last = out.last().map(|x| x.0).or(self.last);
                if matches!(last, Some(l) if l != ' ') {
                    out.push((' ', 1.0));
                }
                if let Some(l) = out.last() {
                    self.last = Some(l.0);
                }
                continue;
            }
            let alts = up.committed_alts.get(i).cloned().unwrap_or_default();
            let strength = up.committed_strength.get(i).copied().unwrap_or(99.0);
            if self.enabled {
                self.word.push(Pending { c, conf, alts, strength });
            } else {
                out.push((c, conf));
            }
        }
        if let Some(l) = out.last() {
            self.last = Some(l.0);
        }
        out
    }

    /// End of the signal: the word in progress becomes final.
    pub fn flush(&mut self) -> Vec<(char, f32)> {
        let out = self.finish_word();
        if let Some(l) = out.last() {
            self.last = Some(l.0);
        }
        out
    }

    /// The word being held back, shown as tentative text meanwhile.
    /// Forgets the word being built (the decoder moved to another signal).
    pub fn discard(&mut self) {
        self.word.clear();
        self.last = None;
    }

    pub fn held(&self) -> Vec<TentativeChar> {
        self.word.iter().map(|p| TentativeChar { best: p.c, confidence: p.conf, alts: p.alts.clone() }).collect()
    }

    fn finish_word(&mut self) -> Vec<(char, f32)> {
        let word = std::mem::take(&mut self.word);
        if word.is_empty() {
            return Vec::new();
        }
        // Options per position: the reading itself, plus alternatives where it is unsure.
        let mut order: Vec<usize> = (0..word.len()).filter(|&i| word[i].conf < UNSURE && !word[i].alts.is_empty()).collect();
        order.sort_by(|a, b| word[*a].conf.partial_cmp(&word[*b].conf).unwrap());
        let mut options: Vec<Vec<(char, f32)>> = word.iter().map(|p| vec![(p.c, p.conf.max(1e-4))]).collect();
        let mut combos = 1usize;
        for &i in &order {
            let extra: Vec<(char, f32)> = word[i].alts.iter().take(2).cloned().collect();
            if combos * (1 + extra.len()) > MAX_COMBOS {
                break;
            }
            combos *= 1 + extra.len();
            options[i].extend(extra);
        }
        let mut best: Option<(f32, Vec<(char, f32)>)> = None;
        let mut idx = vec![0usize; word.len()];
        loop {
            let chosen: Vec<(char, f32)> = idx.iter().enumerate().map(|(i, &k)| options[i][k]).collect();
            let text: String = chosen.iter().map(|x| x.0).collect();
            let score: f32 = chosen.iter().map(|x| x.1.max(1e-4).ln()).sum::<f32>() + self.model.score(&text);
            if best.as_ref().map_or(true, |b| score > b.0) {
                best = Some((score, chosen));
            }
            // Next combination.
            let mut p = 0;
            loop {
                if p == idx.len() {
                    break;
                }
                idx[p] += 1;
                if idx[p] < options[p].len() {
                    break;
                }
                idx[p] = 0;
                p += 1;
            }
            if p == idx.len() {
                break;
            }
        }
        let chosen = best.map(|b| b.1).unwrap_or_default();
        let text: String = chosen.iter().map(|x| x.0).collect();
        if chosen.iter().all(|x| x.1 >= 0.3) {
            self.model.heard(&text);
        }
        // Junk: one or two single-mark characters that hardly stood out of the noise.
        let single_mark = |c: char| matches!(c, 'E' | 'T');
        let short_marks = |c: char| matches!(c, 'E' | 'T' | 'I' | 'M' | 'N' | 'A');
        let weak = word.iter().all(|p| p.strength < self.junk_strength);
        if !self.model.known(&text) && weak && (text.chars().count() == 1 && text.chars().all(single_mark) || text.chars().count() == 2 && text.chars().all(short_marks)) {
            return Vec::new();
        }
        chosen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn up(text: &str, conf: &[f32], alts: Vec<Vec<(char, f32)>>, strength: f32) -> DecodeUpdate {
        DecodeUpdate {
            committed: text.to_string(),
            committed_conf: conf.to_vec(),
            committed_alts: alts,
            committed_strength: vec![strength; text.chars().count()],
            ..Default::default()
        }
    }

    #[test]
    fn callsign_shapes() {
        for c in ["ON3VZ", "DL1ABC", "F5XYZ", "9A1A", "ON3VZ/P", "PA/ON3VZ", "K1A"] {
            assert!(looks_like_callsign(c), "{c}");
        }
        for c in ["CQ", "599", "TNX", "ON3", "3VZ", "ONNN3VZZZZZ"] {
            assert!(!looks_like_callsign(c), "{c}");
        }
    }

    #[test]
    fn an_unsure_character_becomes_a_known_word() {
        let mut wc = WordCorrector::new(true);
        // "QRG" read as "QRZ"? Here: "QTH" read as "QTI" with H as a close second.
        let u = up("QTI ", &[1.0, 1.0, 0.55, 1.0], vec![vec![], vec![], vec![('H', 0.4)], vec![]], 9.0);
        let out: String = wc.push(&u).iter().map(|x| x.0).collect();
        assert_eq!(out, "QTH ");
    }

    #[test]
    fn a_sure_word_is_never_changed() {
        let mut wc = WordCorrector::new(true);
        let u = up("HANS ", &[1.0, 1.0, 1.0, 1.0, 1.0], vec![vec![]; 5], 9.0);
        let out: String = wc.push(&u).iter().map(|x| x.0).collect();
        assert_eq!(out, "HANS ");
    }

    #[test]
    fn weak_single_e_is_dropped_but_a_strong_one_stays() {
        let mut wc = WordCorrector::new(true);
        let out: String = wc.push(&up("E ", &[1.0, 1.0], vec![vec![]; 2], 1.0)).iter().map(|x| x.0).collect();
        assert_eq!(out, "");
        let out: String = wc.push(&up("E ", &[1.0, 1.0], vec![vec![]; 2], 8.0)).iter().map(|x| x.0).collect();
        assert_eq!(out, "E ");
    }

    #[test]
    fn known_short_words_survive_even_when_weak() {
        let mut wc = WordCorrector::new(true);
        let out: String = wc.push(&up("K ", &[1.0, 1.0], vec![vec![]; 2], 1.0)).iter().map(|x| x.0).collect();
        assert_eq!(out, "K ");
    }

    #[test]
    fn disabled_passes_through() {
        let mut wc = WordCorrector::new(false);
        let out: String = wc.push(&up("QTI ", &[1.0, 1.0, 0.55, 1.0], vec![vec![], vec![], vec![('H', 0.4)], vec![]], 1.0)).iter().map(|x| x.0).collect();
        assert_eq!(out, "QTI ");
    }

    #[test]
    fn known_words_numbers_and_unrelated_words_are_left_alone() {
        let mut w = WordCorrector::new(true);
        let out: String = w.push(&up("IS 55 HANS TEST ", &[1.0; 16], vec![vec![]; 16], 99.0)).iter().map(|x| x.0).collect();
        assert_eq!(out, "IS 55 HANS TEST ");
    }
}
