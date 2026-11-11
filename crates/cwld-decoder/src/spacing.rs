//! Where do the words end? The decoder reads characters and measures the silence before each
//! one. This stage decides which of those silences are word gaps.
//!
//! A fixed rule (more than about five units is a word gap) fails on real operators: one spells
//! a name with the letters far apart, the next runs CQ and DE together. So the rule here has
//! three parts:
//!
//! 1. The threshold follows the sender: the middle between his usual gap inside a word and his
//!    usual gap between words, learned from the gaps decided so far.
//! 2. A few characters are held back, so a gap is judged with what comes after it as well.
//! 3. Word knowledge breaks ties: `CQ DE` is two words even when keyed as one, and four loose
//!    letters `A L D O` are almost never four words. Timing that is clear always wins.

use crate::decoder::{DecodeUpdate, TentativeChar};
use crate::lang::{looks_like_callsign, WordModel};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
struct Tok {
    c: char,
    conf: f32,
    alts: Vec<(char, f32)>,
    strength: f32,
    /// Silence before this character, in units (dot lengths).
    gap: f32,
    /// Gaps between this character's own elements, in units.
    intra: Vec<f32>,
}

/// Word gap threshold in units before anything is learned: between 3 (characters) and 7 (words).
const THR_START: f32 = 4.8;
const THR_MIN: f32 = 3.6;
const THR_MAX: f32 = 16.0;
/// Characters kept back so a gap is judged with what follows it.
const LOOK: usize = 5;
/// Longest queue before a prefix is let go without a word end in it.
const MAX_QUEUE: usize = 24;
const MAX_WORD: usize = 16;
/// How steeply the timing speaks: nats per natural-log step away from the threshold.
const SLOPE: f32 = 8.0;
/// Below this share of the threshold a gap is never a word gap, above `SURE_WORD` always.
const SURE_CHAR: f32 = 0.62;
const SURE_WORD: f32 = 2.2;
/// What it costs (nats) to assume that one dot or dash was lost in a fade.
const LOST_ELEMENT: f32 = 2.0;
/// The same towards a common CW word (CQ, 599, TNX): a little more, a word proves less than a
/// callsign that was just heard.
const LOST_IN_WORD: f32 = 1.5;

/// Words that stand alone and are never the start or end of a longer word in a QSO.
const STRONG: &[&str] = &["CQ", "DE", "PSE", "TU", "BK", "QTH", "QRZ", "QSL", "QSO", "RST", "UR", "NAME", "OP", "ES", "TNX", "73", "5NN", "599", "QRS", "QRL", "AGN", "CPY"];

pub struct Spacer {
    pub enabled: bool,
    /// Characters below this mark strength count as possible noise blips (as in the word model).
    pub junk_strength: f32,
    thr: f32,
    /// Gaps decided so far: (units, was a word gap).
    hist: VecDeque<(f32, bool)>,
    queue: Vec<Tok>,
    /// The last gaps inside characters, in units: this sender's weighting.
    intra_hist: VecDeque<f32>,
    /// The decoder's own measure of this sender's gap between characters, 0 when unknown.
    dec_char_gap: f32,
    /// The first queued character continues a word whose start was already let go.
    joined: bool,
}

impl Spacer {
    pub fn new(enabled: bool) -> Self {
        Spacer { enabled, junk_strength: 3.0, thr: THR_START, hist: VecDeque::new(), queue: Vec::new(), intra_hist: VecDeque::new(), dec_char_gap: 0.0, joined: false }
    }

    /// The word gap threshold in units as learned from this sender.
    pub fn threshold(&self) -> f32 {
        self.thr
    }

    /// This sender's usual gap between characters, in units.
    pub fn char_gap(&self) -> f32 {
        let mut v: Vec<f32> = self.hist.iter().filter(|g| !g.1).map(|g| g.0).collect();
        if v.len() < 3 {
            return 3.0;
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    }

    /// Forgets everything (the decoder moved to another signal).
    pub fn discard(&mut self) {
        self.queue.clear();
        self.hist.clear();
        self.intra_hist.clear();
        self.thr = THR_START;
        self.joined = false;
    }

    /// Characters held back, shown as tentative text meanwhile.
    pub fn held(&self) -> Vec<TentativeChar> {
        self.queue.iter().map(|t| TentativeChar { best: t.c, confidence: t.conf, alts: t.alts.clone() }).collect()
    }

    /// Takes the decoder's new characters and gives back what is final now, with the spaces
    /// decided here (the decoder's own spaces are dropped).
    pub fn push(&mut self, up: &DecodeUpdate, model: &WordModel) -> DecodeUpdate {
        if !self.enabled {
            return up.clone();
        }
        self.dec_char_gap = up.char_gap_units;
        for (i, c) in up.committed.chars().enumerate() {
            if c == ' ' {
                continue;
            }
            self.queue.push(Tok {
                c,
                conf: up.committed_conf.get(i).copied().unwrap_or(1.0),
                alts: up.committed_alts.get(i).cloned().unwrap_or_default(),
                strength: up.committed_strength.get(i).copied().unwrap_or(99.0),
                gap: up.committed_gap.get(i).copied().unwrap_or(0.0),
                intra: up.committed_intra.get(i).cloned().unwrap_or_default(),
            });
            for g in &self.queue.last().unwrap().intra {
                self.intra_hist.push_back(*g);
            }
            while self.intra_hist.len() > 40 {
                self.intra_hist.pop_front();
            }
        }
        let mut out = DecodeUpdate { tentative: up.tentative.clone(), wpm: up.wpm, locked: up.locked, silence_units: up.silence_units, ..Default::default() };
        // The decoder found that the sender went from wide to tight spacing (see
        // `Decoder::tightened`). What is held from before the change is settled with the old
        // threshold, the rest and all that follows with one for the new spacing. (Wider
        // spacing needs no such signal: it shows in the gaps held here themselves.)
        if up.spacing_changed && self.dec_char_gap > 0.0 {
            if let Some(i) = (1..self.queue.len()).rev().find(|&i| self.queue[i].gap > 2.0 * self.dec_char_gap) {
                let new = self.queue.split_off(i);
                self.release(model, true, &mut out);
                self.queue = new;
            }
            self.hist.clear();
            self.thr = (1.55 * self.dec_char_gap).clamp(THR_MIN, THR_MAX);
        }
        // A long silence ends the word: everything held is final.
        // Well beyond a word gap: a sender who spaces his letters far apart must not have every
        // letter made final as a word before his spacing is known.
        let quiet = up.silence_units >= (3.2 * self.base_threshold()).max(14.0);
        self.release(model, quiet, &mut out);
        out
    }

    /// End of the signal: everything held becomes final.
    pub fn flush(&mut self, model: &WordModel) -> DecodeUpdate {
        let mut out = DecodeUpdate::default();
        if self.enabled {
            self.release(model, true, &mut out);
        }
        out
    }

    fn release(&mut self, model: &WordModel, all: bool, out: &mut DecodeUpdate) {
        if self.queue.is_empty() || (!all && self.queue.len() <= LOOK) {
            return;
        }
        let (spaces, mended) = self.segment(model, all);
        // How many characters go out: all of them, or up to the last word end that has
        // `LOOK` characters after it.
        let n = self.queue.len();
        let upto = if all {
            n
        } else {
            let settled = n - LOOK;
            match (1..=settled).rev().find(|&i| spaces[i]) {
                Some(i) => i,
                None if n > MAX_QUEUE => settled,
                None => return,
            }
        };
        let ends_word = all || spaces[upto];
        let rest = self.queue.split_off(upto);
        let gone = std::mem::replace(&mut self.queue, rest);
        let mut i = 0;
        while i < gone.len() {
            // The first one starts a word, unless it continues one that was let go before.
            if i > 0 || !self.joined {
                self.learn(gone[i].gap, i == 0 || spaces[i]);
            }
            if i > 0 && spaces[i] {
                push(out, ' ', 1.0, Vec::new(), 99.0, 0.0);
            }
            if let Some((end, text)) = mended.get(&i).filter(|m| m.0 <= gone.len()) {
                // Same elements, other boundaries: as sure as the least sure character was.
                let conf = gone[i..*end].iter().map(|t| t.conf).fold(1.0f32, f32::min).min(0.85);
                let strength = gone[i..*end].iter().map(|t| t.strength).fold(f32::MAX, f32::min);
                for (k, c) in text.chars().enumerate() {
                    push(out, c, conf, Vec::new(), strength, if k == 0 { gone[i].gap } else { 0.0 });
                }
                for t in &gone[i + 1..*end] {
                    self.learn(t.gap, false);
                }
                i = *end;
                continue;
            }
            let t = &gone[i];
            push(out, t.c, t.conf, t.alts.clone(), t.strength, t.gap);
            i += 1;
        }
        if ends_word {
            push(out, ' ', 1.0, Vec::new(), 99.0, 0.0);
        }
        self.joined = !ends_word;
    }

    /// Remembers a decided gap and moves the threshold to the middle between this sender's
    /// usual gap inside a word and his usual gap between words.
    fn learn(&mut self, gap: f32, space: bool) {
        // A pause is a word gap, but says nothing about how long this sender's word gaps are.
        if gap < 1.2 || gap > 3.0 * self.thr.max(self.base_threshold()) {
            return;
        }
        self.hist.push_back((gap, space));
        while self.hist.len() > 20 {
            self.hist.pop_front();
        }
        let median = |mut v: Vec<f32>| {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            v[v.len() / 2]
        };
        let chars: Vec<f32> = self.hist.iter().filter(|g| !g.1).map(|g| g.0).collect();
        let words: Vec<f32> = self.hist.iter().filter(|g| g.1).map(|g| g.0).collect();
        if chars.len() < 3 {
            return;
        }
        let cc = median(chars);
        let mut target = 1.55 * cc;
        if words.len() >= 2 {
            let wc = median(words);
            if wc >= 1.5 * cc {
                // Text lost in a fade leaves long gaps that are no word gaps: never more than
                // three times the gap between characters (7 to 3 by the book).
                target = (cc * wc.min(3.0 * cc)).sqrt();
            }
        }
        self.thr = (self.thr.ln() * 0.6 + target.ln() * 0.4).exp().clamp(THR_MIN, THR_MAX);
    }

    /// The threshold for the characters held now. When their own gaps fall in two clear groups
    /// (most of them short, a few long), the middle between the groups counts, whatever was
    /// learned before: this sender, or this word, is spaced differently. A name spelled out
    /// with the letters far apart is the usual case.
    fn local_threshold(&self) -> f32 {
        let base = self.base_threshold();
        let mut g: Vec<f32> = self.queue.iter().skip(1).map(|t| t.gap).filter(|g| *g >= 1.2).collect();
        if g.len() < 4 {
            return base;
        }
        g.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // The widest step between neighbours, with at least half of the gaps below it.
        let mut best: Option<(f32, f32)> = None;
        for i in (g.len() / 2).max(2)..g.len() {
            let ratio = g[i] / g[i - 1];
            // The gaps below the step must be one group without a step of their own. Text lost
            // in a fade leaves long gaps too, but then the real word gap sits below them as a
            // step by itself (3, 3, 6.5 and then 15).
            if (1..i).any(|k| g[k] / g[k - 1] >= 1.45) {
                break;
            }
            if best.map_or(true, |b| ratio > b.0) {
                best = Some((ratio, (g[i] * g[i - 1]).sqrt()));
            }
        }
        match best {
            Some((ratio, mid)) if ratio >= 1.8 => mid.clamp(THR_MIN, THR_MAX),
            // A smaller step counts for half, and only above a tight group of short gaps: a
            // sender whose gaps run from two to six units has no step there, just a spread.
            Some((ratio, mid)) if ratio >= 1.4 && (mid * mid / g[0] / g[0]).sqrt() / ratio.sqrt() <= 1.6 => (mid * base).sqrt().clamp(THR_MIN, THR_MAX),
            _ => base,
        }
    }

    /// The learned threshold, or, while too little of this sender has been decided here, the
    /// one that follows from the spacing the decoder measured in his first characters.
    fn base_threshold(&self) -> f32 {
        if self.hist.iter().filter(|g| !g.1).count() >= 3 || self.dec_char_gap <= 0.0 {
            self.thr
        } else {
            (1.55 * self.dec_char_gap).clamp(THR_MIN, THR_MAX)
        }
    }

    fn word_score(&self, model: &WordModel, w: &str) -> f32 {
        let len = w.chars().count();
        if STRONG.contains(&w) {
            return 2.5;
        }
        if model.heard_before(w) {
            // A station repeats its callsign: the one heard a moment ago is the likely one.
            return 6.0;
        }
        if len == 1 {
            // A lone letter is rarely a word (K, R and a few more are); a lone digit often is.
            return if model.known(w) { 0.3 } else if w.chars().all(|c| c.is_ascii_digit()) { 0.0 } else { -1.5 };
        }
        if model.known(w) {
            return (0.8 + 0.35 * len as f32).min(3.0);
        }
        if looks_like_callsign(w) {
            return 2.5;
        }
        let rst = len == 3 && w.chars().enumerate().all(|(i, c)| if i == 0 { ('1'..='5').contains(&c) } else { c.is_ascii_digit() || c == 'N' });
        if rst || w.chars().all(|c| c.is_ascii_digit()) {
            return 1.5;
        }
        // A short unknown piece is more often the crumb of a wrong split (BE RLIN, LAPT OP)
        // than a word of its own.
        if (2..=4).contains(&len) {
            return -0.8;
        }
        0.0
    }

    /// The boundary between a gap inside a character and a gap between characters for this
    /// sender, in units: the middle between his two usual gaps (1 and 3 by the book).
    fn element_threshold(&self) -> f32 {
        let intra = self.intra_gap();
        (intra * self.char_gap().max(1.6 * intra)).sqrt()
    }

    /// Another reading of characters i..j with the same dots and dashes: a known word or a
    /// callsign heard before, when moving the boundaries costs less than the word is worth.
    /// A boundary that the timing shows clearly (a gap near one unit, or near three) is
    /// expensive to move, a gap of about two units is cheap.
    fn mend(&self, model: &WordModel, i: usize, j: usize, text: &str, own: f32, word_thr: f32) -> Option<(String, f32)> {
        // The elements of the span and the gap after each one except the last.
        let mut el = String::new();
        let mut gaps: Vec<(f32, bool)> = Vec::new();
        for (k, t) in self.queue[i..j].iter().enumerate() {
            let pat = cwld_types::morse::pattern(t.c)?;
            if t.intra.len() + 1 != pat.len() {
                return None;
            }
            if k > 0 {
                gaps.push((t.gap, true));
            }
            gaps.extend(t.intra.iter().map(|g| (*g, false)));
            el.push_str(pat);
        }
        let thr = self.element_threshold();
        let mut best: Option<(String, f32)> = None;
        for w in model.same_elements(&el) {
            if w == text {
                continue;
            }
            // Where this word has its character boundaries, counted in elements.
            let mut ends = std::collections::HashSet::new();
            let mut at = 0;
            for c in w.chars() {
                at += cwld_types::morse::pattern(c).map_or(0, |p| p.len());
                ends.insert(at);
            }
            let mut score = self.word_score(model, w);
            for (k, (g, was_char)) in gaps.iter().enumerate() {
                let is_char = ends.contains(&(k + 1));
                if is_char != *was_char {
                    let odds = (SLOPE * (g.max(0.2) / thr).ln()).clamp(-8.0, 8.0);
                    score += if is_char { odds } else { -odds };
                }
            }
            if score > own && best.as_ref().map_or(true, |b| score > b.1) {
                best = Some((w.to_string(), score));
            }
        }
        // One dot or dash lost in a fade: K (dash dot dash) with its dot gone reads as T T.
        // Only towards a callsign heard a moment ago or a common CW word, and only where the
        // silence is as long as the lost element with the gaps around it would be. A callsign
        // that merely looks like another one has no such hole and is left alone.
        let intra = self.intra_gap();
        let char_gap = self.char_gap().max(1.6 * intra);
        let calls = model.recent_calls().map(|w| (w, LOST_ELEMENT));
        let words = model.words_with_elements(el.len() + 1).map(|w| (w, LOST_IN_WORD));
        for (w, cost) in calls.chain(words) {
            let Some(wel) = crate::lang::elements(w) else { continue };
            if w == text || wel.len() != el.len() + 1 {
                continue;
            }
            let mut ends = std::collections::HashSet::new();
            let mut at = 0;
            for c in w.chars() {
                at += cwld_types::morse::pattern(c).map_or(0, |p| p.len());
                ends.insert(at);
            }
            let (wb, eb) = (wel.as_bytes(), el.as_bytes());
            // The lost element is number p of the callsign. Not the first or the last one: there
            // the silence next to it is a word gap and shows nothing.
            for p in 1..eb.len() {
                if wb[..p] != eb[..p] || wb[p + 1..] != eb[p..] {
                    continue;
                }
                let lost = if wb[p] == b'-' { 3.0 } else { 1.0 };
                let before = if ends.contains(&p) { char_gap } else { intra };
                let after = if ends.contains(&(p + 1)) { char_gap } else { intra };
                let (room, was_char) = gaps[p - 1];
                let mut score = self.word_score(model, w) - cost - SLOPE * (room / (before + lost + after)).ln().abs();
                if was_char {
                    // The hole was read as a long gap between characters and charged as such
                    // by the word split. It is no gap, so that charge is taken back.
                    score += (SLOPE * (room.max(0.3) / word_thr).ln()).clamp(0.0, 8.0) / 2.0;
                }
                for (k, (g, was_char)) in gaps.iter().enumerate() {
                    if k == p - 1 {
                        continue;
                    }
                    // Gap k of what was read is gap k (before the hole) or k + 1 of the callsign.
                    let is_char = ends.contains(&(if k < p - 1 { k + 1 } else { k + 2 }));
                    if is_char != *was_char {
                        let odds = (SLOPE * (g.max(0.2) / thr).ln()).clamp(-8.0, 8.0);
                        score += if is_char { odds } else { -odds };
                    }
                }
                if score > own && best.as_ref().map_or(true, |b| score > b.1) {
                    best = Some((w.to_string(), score));
                }
            }
        }
        best
    }

    /// This sender's usual gap inside a character, in units.
    fn intra_gap(&self) -> f32 {
        let mut v: Vec<f32> = self.intra_hist.iter().cloned().collect();
        if v.len() < 6 {
            return 1.0;
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2].clamp(0.6, 2.2)
    }

    /// The best split of the queue into words. `spaces[i]` is true when a word gap comes before
    /// character `i` (`spaces[n]`: after the last one). The map gives, per word start, the end
    /// and the mended text of words that are read with other character boundaries.
    fn segment(&self, model: &WordModel, complete: bool) -> (Vec<bool>, std::collections::HashMap<usize, (usize, String)>) {
        let n = self.queue.len();
        const NEG: f32 = f32::NEG_INFINITY;
        // Timing: log odds that the gap before character k is a word gap.
        let known = self.hist.iter().filter(|g| !g.1).count() >= 3 || self.dec_char_gap > 0.0;
        let sure_word = if known { SURE_WORD } else { 3.0 };
        let thr = self.local_threshold();
        let mut odds: Vec<f32> = self.queue.iter().map(|t| (SLOPE * (t.gap.max(0.3) / thr).ln()).clamp(-8.0, 8.0)).collect();
        // Four or more gaps in a row of about the same length are gaps between characters,
        // however long they are: nobody sends three one-letter words in a row, but anybody may
        // start to spell a word slowly. Without this the first slowly spelled word after
        // normal keying came out as loose letters.
        let mut k = 1;
        while k < n {
            let (mut lo, mut hi, mut m) = (self.queue[k].gap, self.queue[k].gap, k);
            while m + 1 < n {
                let g = self.queue[m + 1].gap;
                if lo.min(g) <= 0.0 || hi.max(g) / lo.min(g) > 1.45 {
                    break;
                }
                lo = lo.min(g);
                hi = hi.max(g);
                m += 1;
            }
            if m - k + 1 >= 4 && lo > thr {
                for o in &mut odds[k..=m] {
                    *o = o.min(-2.0);
                }
            }
            k = m + 1;
        }
        let odds = odds;
        let as_space = |k: usize| if self.queue[k].gap < SURE_CHAR * thr { NEG } else { odds[k] / 2.0 };
        let as_char = |k: usize| if self.queue[k].gap >= sure_word * thr && odds[k] > 0.0 { NEG } else { -odds[k] / 2.0 };
        let mut best = vec![NEG; n + 1];
        let mut from = vec![0usize; n + 1];
        let mut alt: Vec<Option<String>> = vec![None; n + 1];
        best[0] = 0.0;
        for j in 1..=n {
            let mut inner = 0.0f32;
            let mut text = String::new();
            for i in (j.saturating_sub(MAX_WORD)..j).rev() {
                // The word is characters i..j. Going down, character i joins at the front.
                text.insert(0, self.queue[i].c);
                if i + 1 < j {
                    inner += as_char(i + 1);
                }
                if inner == NEG {
                    break;
                }
                if best[i] == NEG {
                    continue;
                }
                let boundary = if i == 0 { 0.0 } else { as_space(i) };
                // A word whose start was let go, or whose end is still to come, is not judged.
                let open = (i == 0 && self.joined) || (j == n && !complete);
                // A lone weak mark is probably noise: it is left alone, so the word model can
                // drop it, and never glued to the word next to it to avoid a lone letter.
                let blip = j - i == 1 && self.queue[i].strength < self.junk_strength && matches!(self.queue[i].c, 'E' | 'T');
                let mut word = if open || blip { 0.0 } else { self.word_score(model, &text) };
                // A short word such as OP or DE right after a clear gap between characters is
                // more likely the end of a longer word (LAPTOP): then it counts as an ordinary
                // known word, not as one that always stands alone.
                if i > 0 && odds[i] < -1.5 && j - i <= 2 && STRONG.contains(&text.as_str()) {
                    word = word.min(0.8 + 0.35 * (j - i) as f32);
                }
                let mut other = None;
                if !open && !model.known(&text) && !model.heard_before(&text) {
                    // A reading is not mended just because a lone letter or a short unknown word scores
                    // below nothing: the other word has to be worth it by itself.
                    if let Some((w, sc)) = self.mend(model, i, j, &text, word.max(0.0), thr) {
                        word = sc;
                        other = Some(w);
                    }
                }
                let s = best[i] + boundary + inner + word;
                if s > best[j] {
                    best[j] = s;
                    from[j] = i;
                    alt[j] = other;
                }
            }
        }
        let mut out = vec![false; n + 1];
        let mut mended = std::collections::HashMap::new();
        out[n] = complete;
        let mut j = n;
        while j > 0 {
            let i = from[j];
            if i > 0 {
                out[i] = true;
            }
            if let Some(w) = alt[j].take() {
                mended.insert(i, (j, w));
            }
            j = i;
        }
        (out, mended)
    }
}

fn push(out: &mut DecodeUpdate, c: char, conf: f32, alts: Vec<(char, f32)>, strength: f32, gap: f32) {
    out.committed.push(c);
    out.committed_conf.push(conf);
    out.committed_alts.push(alts);
    out.committed_strength.push(strength);
    out.committed_gap.push(gap);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feeds words with the given gaps: (text, gap before each character in units).
    fn run(tokens: &[(char, f32)]) -> String {
        let model = WordModel::default();
        let mut sp = Spacer::new(true);
        let mut text = String::new();
        for (c, g) in tokens {
            let up = DecodeUpdate { committed: c.to_string(), committed_conf: vec![1.0], committed_alts: vec![vec![]], committed_strength: vec![99.0], committed_gap: vec![*g], ..Default::default() };
            text.push_str(&sp.push(&up, &model).committed);
        }
        text.push_str(&sp.flush(&model).committed);
        text.trim().to_string()
    }

    fn keyed(text: &str, char_gap: f32, word_gap: f32) -> Vec<(char, f32)> {
        let mut out = Vec::new();
        for (wi, w) in text.split(' ').enumerate() {
            for (ci, c) in w.chars().enumerate() {
                out.push((c, if ci > 0 { char_gap } else if wi > 0 { word_gap } else { 50.0 }));
            }
        }
        out
    }

    #[test]
    fn standard_spacing_is_kept() {
        let t = "CQ CQ DE ON3VZ ON3VZ K";
        assert_eq!(run(&keyed(t, 3.0, 7.0)), t);
        for t in ["R TNX FER CALL OM NAME IS HANS QTH BERLIN", "WX COLD AND RAIN TEMP 4 C", "TU 5NN 14 ON6WL", "CQ CQ DE ON3VZ TU 73 E E"] {
            assert_eq!(run(&keyed(t, 3.0, 7.0)), t);
        }
    }

    #[test]
    fn letters_far_apart_stay_one_word() {
        let t = "OP IS ALDO ALDO ALDO ES QTH IS POREC";
        assert_eq!(run(&keyed(t, 6.5, 15.0)), t);
        assert_eq!(run(&keyed(t, 9.0, 21.0)), t);
    }

    #[test]
    fn cq_and_de_keyed_as_one_are_two_words() {
        let mut k = keyed("CQ CQ DE ON3VZ K", 3.0, 7.0);
        k[4].1 = 3.6; // the D of DE right after CQ
        assert_eq!(run(&k), "CQ CQ DE ON3VZ K");
    }

    #[test]
    fn short_word_gaps_are_still_word_gaps() {
        let t = "TNX FER CALL UR RST 599 599 BK";
        assert_eq!(run(&keyed(t, 2.8, 4.6)), t);
    }

    #[test]
    fn an_unknown_word_is_not_cut_into_known_pieces() {
        // TODAY holds TO, ANTWERP holds ANT: clear character gaps keep them whole.
        let t = "QTH ANTWERP TODAY";
        assert_eq!(run(&keyed(t, 3.0, 7.0)), t);
    }

    /// Characters with the gap before each and the gaps inside them, all in units.
    fn run_marks(tokens: &[(char, f32, &[f32])], model: &mut WordModel) -> String {
        run_styled(tokens, model, 0.0)
    }

    /// The same, with the gap between characters the decoder measured for this sender.
    fn run_styled(tokens: &[(char, f32, &[f32])], model: &mut WordModel, char_gap: f32) -> String {
        let mut sp = Spacer::new(true);
        let mut text = String::new();
        for (c, g, intra) in tokens {
            let up = DecodeUpdate { committed: c.to_string(), committed_conf: vec![1.0], committed_alts: vec![vec![]], committed_strength: vec![99.0], committed_gap: vec![*g], committed_intra: vec![intra.to_vec()], char_gap_units: char_gap, ..Default::default() };
            text.push_str(&sp.push(&up, model).committed);
        }
        text.push_str(&sp.flush(model).committed);
        text.trim().to_string()
    }

    #[test]
    fn a_character_cut_in_two_by_a_pause_is_mended_to_a_known_word() {
        let mut m = WordModel::default();
        // 599 with the 5 keyed as S I (a pause of 1.9 units inside it).
        let t: &[(char, f32, &[f32])] = &[('U', 50.0, &[1.0, 1.0]), ('R', 3.0, &[1.0, 1.0]), ('S', 7.0, &[1.0, 1.0]), ('I', 1.9, &[1.0]), ('9', 3.0, &[1.0; 4]), ('9', 3.0, &[1.0; 4]), ('B', 7.0, &[1.0; 3]), ('K', 3.0, &[1.0; 2])];
        assert_eq!(run_marks(t, &mut m), "UR 599 BK");
        // With a clear gap of three units between S and I it stays what was keyed.
        let t: &[(char, f32, &[f32])] = &[('U', 50.0, &[1.0, 1.0]), ('R', 3.0, &[1.0, 1.0]), ('S', 7.0, &[1.0, 1.0]), ('I', 3.0, &[1.0]), ('9', 3.0, &[1.0; 4]), ('9', 3.0, &[1.0; 4]), ('B', 7.0, &[1.0; 3]), ('K', 3.0, &[1.0; 2])];
        assert_eq!(run_marks(t, &mut m), "UR SI99 BK");
    }

    #[test]
    fn two_characters_run_together_are_mended_to_a_known_word() {
        let mut m = WordModel::default();
        // TNX with T and N run together (gap 1.8): read as G X.
        let t: &[(char, f32, &[f32])] = &[('R', 50.0, &[1.0, 1.0]), ('G', 7.0, &[1.8, 1.0]), ('X', 3.0, &[1.0; 3]), ('F', 7.0, &[1.0; 3]), ('E', 3.0, &[]), ('R', 3.0, &[1.0; 2])];
        assert_eq!(run_marks(t, &mut m), "R TNX FER");
    }

    #[test]
    fn a_callsign_heard_before_is_mended_but_a_new_one_is_left_alone() {
        let r: (char, f32, &[f32]) = ('R', 9.0, &[1.9, 1.9]);
        let rest: [(char, f32, &[f32]); 3] = [('M', 5.0, &[1.9]), ('3', 6.0, &[1.9; 4]), ('Y', 5.0, &[1.9; 3])];
        let cut: [(char, f32, &[f32]); 2] = [('E', 9.0, &[]), ('N', 3.6, &[1.9])];
        let mut t = vec![r];
        t.extend(rest);
        t.extend(cut);
        t.extend(rest);
        let mut m = WordModel::default();
        // Not heard before: both stay as read.
        assert_eq!(run_styled(&t, &mut m, 5.3), "RM3Y ENM3Y");
        m.heard("RM3Y");
        assert_eq!(run_styled(&t, &mut m, 5.3), "RM3Y RM3Y");
    }

    #[test]
    fn clear_timing_is_never_overruled() {
        // HANS and SIGS are the same row of dots and dashes; SIGS is a known word.
        let mut m = WordModel::default();
        let t: &[(char, f32, &[f32])] = &[('H', 50.0, &[1.0; 3]), ('A', 3.0, &[1.0]), ('N', 3.0, &[1.0]), ('S', 3.0, &[1.0; 2])];
        assert_eq!(run_marks(t, &mut m), "HANS");
    }

    #[test]
    fn a_dot_lost_in_a_fade_is_put_back_in_a_callsign_heard_before() {
        // IK1QHB, then again with the dot of the K gone: dash, three units of silence, dash.
        let call = |k: &[(char, f32, &'static [f32])]| -> Vec<(char, f32, &'static [f32])> {
            let mut v: Vec<(char, f32, &'static [f32])> = vec![('I', 7.0, &[1.0])];
            v.extend_from_slice(k);
            v.extend_from_slice(&[('1', 3.0, &[1.0; 4]), ('Q', 3.0, &[1.0; 3]), ('H', 3.0, &[1.0; 3]), ('B', 3.0, &[1.0; 3])]);
            v
        };
        let good = call(&[('K', 3.0, &[1.0, 1.0])]);
        let faded = call(&[('T', 3.0, &[]), ('T', 3.0, &[])]);
        let mut both = good.clone();
        both.extend(faded.clone());
        let mut m = WordModel::default();
        // Not heard before: nothing to go by, it stays as read.
        assert_eq!(run_marks(&faded, &mut m), "ITT1QHB");
        m.heard("IK1QHB");
        assert_eq!(run_marks(&both, &mut m), "IK1QHB IK1QHB");
        // T T with a gap that is too long for one lost dot stays.
        let wide = call(&[('T', 3.0, &[]), ('T', 4.3, &[])]);
        assert!(!run_marks(&wide, &mut m).contains("IK1QHB"));
    }

    #[test]
    fn a_callsign_that_only_looks_like_one_heard_before_is_left_alone() {
        // ON3VS was heard, ON3VI calls: one dot fewer, but no hole where that dot could be.
        let mut m = WordModel::default();
        m.heard("ON3VS");
        let t: &[(char, f32, &[f32])] = &[('O', 7.0, &[1.0; 2]), ('N', 3.0, &[1.0]), ('3', 3.0, &[1.0; 4]), ('V', 3.0, &[1.0; 3]), ('I', 3.0, &[1.0]), ('K', 7.0, &[1.0; 2])];
        assert_eq!(run_marks(t, &mut m), "ON3VI K");
        // DL1AD heard, DL1AN calls (D is dash dot dot, N is dash dot).
        let mut m = WordModel::default();
        m.heard("DL1AD");
        let t: &[(char, f32, &[f32])] = &[('D', 7.0, &[1.0; 2]), ('L', 3.0, &[1.0; 3]), ('1', 3.0, &[1.0; 4]), ('A', 3.0, &[1.0]), ('N', 3.0, &[1.0])];
        assert_eq!(run_marks(t, &mut m), "DL1AN");
    }

    #[test]
    fn a_word_spelled_slowly_after_normal_keying_stays_one_word() {
        let mut k = keyed("UR RST 599 IN", 3.0, 7.0);
        k.extend(keyed("X HOBOKEN HW CPY", 8.0, 18.0).into_iter().skip(1));
        assert_eq!(run(&k), "UR RST 599 IN HOBOKEN HW CPY");
        // Three one-letter words in a row at normal spacing are still three words.
        assert_eq!(run(&keyed("CQ CQ DE ON3VZ TEMP 4 C R TU", 3.0, 7.0)), "CQ CQ DE ON3VZ TEMP 4 C R TU");
    }

    #[test]
    fn a_long_silence_releases_the_text_at_once() {
        let model = WordModel::default();
        let mut sp = Spacer::new(true);
        let up = DecodeUpdate { committed: "K".into(), committed_conf: vec![1.0], committed_alts: vec![vec![]], committed_strength: vec![99.0], committed_gap: vec![7.0], ..Default::default() };
        assert_eq!(sp.push(&up, &model).committed, "");
        let quiet = DecodeUpdate { silence_units: 16.0, ..Default::default() };
        assert_eq!(sp.push(&quiet, &model).committed, "K ");
    }
}
