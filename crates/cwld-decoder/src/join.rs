//! Spelled words: a sender who keys `D L 4 S A V` with a word gap after every letter meant
//! `DL4SAV`. The spacing stage (`spacing`) already joins letters when the gaps around them
//! say so; this stage handles what is left, where the gaps are as long as real word gaps.
//!
//! Loose single characters are held back until a longer word (or a long silence) ends the run.
//! The run is then joined when the result is a callsign, a known CW word, or at least three
//! letters with a vowel that are not all the same (a spelled name or QTH). `R R R`, `E E` and a
//! lone `K` stay as they are. As MRP40's maker says of his own version: this is no exact
//! science, so the operator can switch it off.

use crate::decoder::TentativeChar;
use crate::lang::{looks_like_callsign, WordModel};

/// Longest run held back; a longer run is decided at once.
const MAX_RUN: usize = 12;
/// Silence (in units) after which a held run is decided without waiting for the next word.
pub const RELEASE_UNITS: f32 = 20.0;

#[derive(Default)]
pub struct Joiner {
    pub enabled: bool,
    /// Characters of the word being received (not yet ended by a space).
    cur: Vec<(char, f32)>,
    /// Loose single characters held back, in order.
    run: Vec<(char, f32)>,
}

/// Should this run of loose characters be one word?
pub fn should_join(run: &str, model: &WordModel) -> bool {
    let n = run.chars().count();
    if n < 2 {
        return false;
    }
    let first = run.chars().next().unwrap();
    if run.chars().all(|c| c == first) {
        // R R R, E E, 5 5: repeated on purpose.
        return false;
    }
    if looks_like_callsign(run) || model.known(run) {
        return true;
    }
    n >= 3 && run.chars().all(|c| c.is_ascii_alphabetic()) && run.chars().any(|c| matches!(c, 'A' | 'E' | 'I' | 'O' | 'U' | 'Y'))
}

/// Which part of a run becomes one word: `run[lead..mid]` is joined, the rest stays loose
/// (`mid == lead` when nothing is joined; `tail` is always the run length). A `R` before or a
/// `K` or `R` after a spelled callsign or known word is a word of its own: `D L 4 S A V K` is
/// `DL4SAV K`, not the callsign `DL4SAVK`.
fn split_run(run: &[(char, f32)], model: &WordModel) -> (usize, usize, usize) {
    let n = run.len();
    let text: String = run.iter().map(|x| x.0).collect();
    let solid = |t: &str| t.chars().count() >= 3 && (looks_like_callsign(t) || model.known(t));
    let ends_alone = |c: char| matches!(c, 'K' | 'R');
    let mut lead = 0;
    let mut end = n;
    if n >= 4 && run[0].0 == 'R' && solid(&text[1..]) {
        lead = 1;
    }
    if end - lead >= 4 && ends_alone(run[end - 1].0) && solid(&text[lead..end - 1]) {
        end -= 1;
    }
    if should_join(&text[lead..end], model) {
        (lead, end, n)
    } else {
        (0, 0, n)
    }
}

impl Joiner {
    pub fn new(enabled: bool) -> Self {
        Joiner { enabled, ..Default::default() }
    }

    /// Takes final text (words each followed by a space) and gives what is final after joining.
    pub fn push(&mut self, input: &[(char, f32)], model: &WordModel) -> Vec<(char, f32)> {
        if !self.enabled {
            return input.to_vec();
        }
        let mut out = Vec::new();
        for &(c, p) in input {
            if c == ' ' {
                let word = std::mem::take(&mut self.cur);
                self.word_done(word, model, &mut out);
            } else {
                self.cur.push((c, p));
            }
        }
        out
    }

    fn word_done(&mut self, word: Vec<(char, f32)>, model: &WordModel, out: &mut Vec<(char, f32)>) {
        if word.is_empty() {
            return;
        }
        if word.len() == 1 && word[0].0.is_ascii_alphanumeric() {
            self.run.push(word[0]);
            if self.run.len() >= MAX_RUN {
                self.release(model, out);
            }
            return;
        }
        self.release(model, out);
        out.extend(word);
        out.push((' ', 1.0));
    }

    /// Decides the held run and hands it out.
    pub fn release(&mut self, model: &WordModel, out: &mut Vec<(char, f32)>) {
        let run = std::mem::take(&mut self.run);
        if run.is_empty() {
            return;
        }
        let (lead, mid, tail) = split_run(&run, model);
        let loose = |xs: &[(char, f32)], out: &mut Vec<(char, f32)>| {
            for &x in xs {
                out.push(x);
                out.push((' ', 1.0));
            }
        };
        loose(&run[..lead], out);
        if mid > lead {
            out.extend_from_slice(&run[lead..mid]);
            out.push((' ', 1.0));
        }
        loose(&run[mid..tail], out);
        loose(&run[tail..], out);
    }

    /// End of the signal: the run and a word without its space become final.
    pub fn flush(&mut self, model: &WordModel) -> Vec<(char, f32)> {
        let mut out = Vec::new();
        let word = std::mem::take(&mut self.cur);
        if !word.is_empty() {
            self.word_done(word, model, &mut out);
        }
        self.release(model, &mut out);
        out
    }

    pub fn discard(&mut self) {
        self.cur.clear();
        self.run.clear();
    }

    pub fn holding(&self) -> bool {
        !self.run.is_empty()
    }

    /// The held characters, shown as tentative text (with their spaces) meanwhile.
    pub fn held(&self) -> Vec<TentativeChar> {
        let mut v = Vec::new();
        for &(c, p) in &self.run {
            v.push(TentativeChar { best: c, confidence: p, alts: Vec::new() });
            v.push(TentativeChar { best: ' ', confidence: 1.0, alts: Vec::new() });
        }
        for &(c, p) in &self.cur {
            v.push(TentativeChar { best: c, confidence: p, alts: Vec::new() });
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> String {
        let m = WordModel::default();
        let mut j = Joiner::new(true);
        let input: Vec<(char, f32)> = text.chars().map(|c| (c, 1.0)).collect();
        let mut out = j.push(&input, &m);
        out.extend(j.flush(&m));
        out.iter().map(|x| x.0).collect::<String>().trim_end().to_string()
    }

    #[test]
    fn spelled_callsigns_names_and_reports_are_joined() {
        assert_eq!(run("DE D L 4 S A V K "), "DE DL4SAV K");
        assert_eq!(run("NAME IS H A N S HANS "), "NAME IS HANS HANS");
        assert_eq!(run("UR 5 N N "), "UR 5NN");
        assert_eq!(run("QTH G E N T"), "QTH GENT");
        assert_eq!(run("R O N 3 V Z K "), "R ON3VZ K");
        assert_eq!(run("NAME E R I K "), "NAME ERIK");
    }

    #[test]
    fn single_letters_that_belong_alone_stay() {
        assert_eq!(run("R R R TU "), "R R R TU");
        assert_eq!(run("73 E E "), "73 E E");
        assert_eq!(run("QSL R 5NN K "), "QSL R 5NN K");
        assert_eq!(run("BK R K "), "BK R K");
        assert_eq!(run("K "), "K");
        assert_eq!(run("1 4 "), "1 4");
    }

    #[test]
    fn off_passes_through() {
        let m = WordModel::default();
        let mut j = Joiner::new(false);
        let input: Vec<(char, f32)> = "D L 4 S A V ".chars().map(|c| (c, 1.0)).collect();
        let out: String = j.push(&input, &m).iter().map(|x| x.0).collect();
        assert_eq!(out, "D L 4 S A V ");
    }

    #[test]
    fn held_letters_show_as_tentative() {
        let m = WordModel::default();
        let mut j = Joiner::new(true);
        let input: Vec<(char, f32)> = "CQ D L ".chars().map(|c| (c, 1.0)).collect();
        let out: String = j.push(&input, &m).iter().map(|x| x.0).collect();
        assert_eq!(out, "CQ ");
        let held: String = j.held().iter().map(|t| t.best).collect();
        assert_eq!(held, "D L ");
        let mut rest = Vec::new();
        j.release(&m, &mut rest);
        assert_eq!(rest.iter().map(|x| x.0).collect::<String>(), "D L ");
    }
}
