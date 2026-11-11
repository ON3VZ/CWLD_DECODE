use cwld_dsp::FRAME_RATE;
use cwld_types::morse;

const NEG: f64 = f64::NEG_INFINITY;

#[derive(Clone, Debug)]
pub struct DecoderConfig {
    pub wpm_min: f64,
    pub wpm_max: f64,
    /// Frames are correlated by the low-pass filter, so their evidence is worth less than
    /// independent samples. This factor scales the evidence against the timing prior.
    pub evidence_scale: f64,
    /// Spread (natural log) of mark and gap durations around their expected value.
    pub mark_sigma: f64,
    pub gap_sigma: f64,
    /// Characters below this confidence are reported as tentative with alternatives.
    pub tentative_below: f32,
    /// Cost (nats) of every key-down segment. Raising it makes the decoder less willing to
    /// read noise blips as dots.
    pub mark_penalty: f64,
    /// A fixed speed in WPM: the decoder does not look for another one.
    pub fixed_wpm: Option<f64>,
    /// Follow the sender's own style: weighting (gap inside a character) and spacing, taken
    /// from his first marks at once and followed after that.
    pub learn_style: bool,
}

impl Default for DecoderConfig {
    fn default() -> Self {
        DecoderConfig { wpm_min: 5.0, wpm_max: 50.0, evidence_scale: 0.1, mark_sigma: 0.28, gap_sigma: 0.30, tentative_below: 0.9, mark_penalty: 0.0, fixed_wpm: None, learn_style: true }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TentativeChar {
    pub best: char,
    pub confidence: f32,
    pub alts: Vec<(char, f32)>,
}

#[derive(Clone, Debug, Default)]
pub struct DecodeUpdate {
    /// New final text since the last call.
    pub committed: String,
    /// One confidence (0..1) per character of `committed`. A space has 1.0.
    pub committed_conf: Vec<f32>,
    /// Per character of `committed`: the other readings with their probability (empty for a
    /// space or a sure character). Used by the word model.
    pub committed_alts: Vec<Vec<(char, f32)>>,
    /// Per character: how strongly the key-down frames of its weakest mark stood out of the
    /// noise (mean evidence). Low values mean the character may be a noise blip. Spaces: 99.
    pub committed_strength: Vec<f32>,
    /// Per character: the silence before it in units (dot lengths). Spaces: 0.
    pub committed_gap: Vec<f32>,
    /// Per character: the gaps between its own elements, in units (one fewer than elements).
    pub committed_intra: Vec<Vec<f32>>,
    /// Silence since the last character, in units, when nothing is waiting in the decoder.
    pub silence_units: f32,
    /// This sender's usual gap between characters in units as measured so far (3 by the book),
    /// 0 while no signal is locked.
    pub char_gap_units: f32,
    /// The sender went from wide to tight spacing since the last poll (see `Decoder::tightened`).
    pub spacing_changed: bool,
    /// Not final yet: the characters of the last stretch with their alternatives.
    pub tentative: Vec<TentativeChar>,
    pub wpm: f32,
    pub locked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Label {
    Dot,
    Dash,
    Intra,
    Char,
    Word,
    Lead,
}

#[derive(Clone, Copy, Debug)]
struct Seg {
    label: Label,
    start: usize,
    end: usize,
    open: bool,
}

#[derive(Clone, Copy, Debug)]
struct Timing {
    /// Unit length in frames.
    u: f64,
    dash: f64,
    gap: f64,
    /// Gap inside a character in units (1 by the book, up to 2 with heavy weighting).
    intra: f64,
}

struct Path {
    score: f64,
    segs: Vec<Seg>,
}

#[derive(Clone, Copy, Default)]
struct Back {
    from: usize,
    label: u8,
}

fn lab(l: u8) -> Label {
    match l {
        0 => Label::Dot,
        1 => Label::Dash,
        2 => Label::Intra,
        3 => Label::Char,
        4 => Label::Word,
        _ => Label::Lead,
    }
}

pub struct Decoder {
    cfg: DecoderConfig,
    llr: Vec<f32>,
    u: Option<f64>,
    dash: f64,
    gap: f64,
    intra: f64,
    /// The last gaps between characters and words, in units: the sender's spacing.
    gap_hist: std::collections::VecDeque<f64>,
    /// The last gaps read as gaps inside characters, in units.
    intra_hist: std::collections::VecDeque<f64>,
    spacing_changed: bool,
    /// Marks learned from so far: the style is followed fast at first, then slowly.
    learned: usize,
    gaps: Vec<f32>,
    intras: Vec<Vec<f32>>,
    /// Silence after the last committed character (frames), for the gap before the next one.
    gap_carry: usize,
    trail: usize,
    locked: bool,
    force_lock: bool,
    since_step: usize,
    since_global: usize,
    committed: String,
    conf: Vec<f32>,
    alts: Vec<Vec<(char, f32)>>,
    strength: Vec<f32>,
    tentative: Vec<TentativeChar>,
    emitted_any: bool,
    space_done: bool,
    /// Frames of silence after the last committed character that are no longer in `llr`.
    carry: usize,
    patterns: Vec<Vec<(char, Vec<bool>)>>,
}

impl Decoder {
    pub fn new(cfg: DecoderConfig) -> Self {
        // All valid morse patterns, grouped by their number of elements (true = dash).
        let mut patterns: Vec<Vec<(char, Vec<bool>)>> = vec![Vec::new(); 9];
        let mut seen = std::collections::HashSet::new();
        for c in ('A'..='Z').chain('0'..='9').chain(".,?/=+-:'\"()@".chars()) {
            if let Some(p) = morse::pattern(c) {
                if seen.insert(p) {
                    let bits: Vec<bool> = p.chars().map(|e| e == '-').collect();
                    patterns[bits.len()].push((c, bits));
                }
            }
        }
        Decoder {
            cfg,
            llr: Vec::new(),
            u: None,
            dash: 3.0,
            gap: 1.0,
            intra: 1.0,
            gap_hist: std::collections::VecDeque::new(),
            intra_hist: std::collections::VecDeque::new(),
            spacing_changed: false,
            learned: 0,
            gaps: Vec::new(),
            intras: Vec::new(),
            gap_carry: 0,
            trail: 0,
            locked: false,
            force_lock: false,
            since_step: 0,
            since_global: usize::MAX / 2,
            committed: String::new(),
            conf: Vec::new(),
            alts: Vec::new(),
            strength: Vec::new(),
            tentative: Vec::new(),
            emitted_any: false,
            space_done: true,
            carry: 0,
            patterns,
        }
    }

    pub fn wpm(&self) -> Option<f64> {
        self.u.map(|u| 240.0 / u)
    }

    /// Changes the speed range, or fixes the speed (`fixed`). Applies from the next search.
    pub fn set_speed_range(&mut self, min: f64, max: f64, fixed: Option<f64>) {
        self.cfg.wpm_min = min;
        self.cfg.wpm_max = max;
        self.cfg.fixed_wpm = fixed;
        if let (Some(w), true) = (fixed, self.u.is_some()) {
            self.u = Some(240.0 / w);
        }
    }

    /// Nothing is being read right now: silence of at least a word gap since the last character.
    pub fn idle(&self) -> bool {
        self.trail as f64 >= 5.0 * self.u.unwrap_or(12.0)
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// Feeds evidence. `p_down` is the probability that the key is down, per frame.
    pub fn push_prob(&mut self, p_down: &[f32]) {
        let l: Vec<f32> = p_down
            .iter()
            .map(|p| {
                let p = p.clamp(1e-4, 1.0 - 1e-4);
                (p / (1.0 - p)).ln()
            })
            .collect();
        self.push_llr(&l);
    }

    /// Feeds log-likelihood ratios (positive: key down is more likely).
    pub fn push_llr(&mut self, llr: &[f32]) {
        self.llr.extend_from_slice(llr);
        self.since_step += llr.len();
        self.since_global = self.since_global.saturating_add(llr.len());
        if self.since_step >= 20 {
            self.since_step = 0;
            self.step();
        }
    }

    /// Declares the end of the signal: long silence, everything left becomes final.
    pub fn flush(&mut self) {
        self.force_lock = true;
        let u = self.u.unwrap_or(12.0);
        let quiet = vec![-8.0f32; (14.0 * u) as usize + 20];
        for chunk in quiet.chunks(20) {
            self.push_llr(chunk);
        }
    }

    pub fn poll(&mut self) -> DecodeUpdate {
        let mut up = DecodeUpdate::default();
        up.committed = std::mem::take(&mut self.committed);
        up.committed_conf = std::mem::take(&mut self.conf);
        up.committed_alts = std::mem::take(&mut self.alts);
        up.committed_strength = std::mem::take(&mut self.strength);
        up.committed_gap = std::mem::take(&mut self.gaps);
        up.committed_intra = std::mem::take(&mut self.intras);
        up.silence_units = self.trail as f32 / self.u.unwrap_or(12.0) as f32;
        up.spacing_changed = std::mem::take(&mut self.spacing_changed);
        up.char_gap_units = if self.locked { (3.0 * self.gap) as f32 } else { 0.0 };
        up.tentative = self.tentative.clone();
        up.wpm = self.wpm().unwrap_or(0.0) as f32;
        up.locked = self.locked;
        up
    }

    // ---------- timing model ----------

    fn exp_dur(t: &Timing, l: Label) -> f64 {
        match l {
            Label::Dot => t.u,
            Label::Intra => t.intra * t.u,
            Label::Dash => t.dash * t.u,
            Label::Char => 3.0 * t.u * t.gap,
            Label::Word => 7.0 * t.u * t.gap,
            Label::Lead => 0.0,
        }
    }

    fn range(t: &Timing, l: Label) -> (usize, usize) {
        let e = Self::exp_dur(t, l);
        let (lo, hi) = match l {
            Label::Dot => (0.4 * e, 2.0 * e),
            Label::Dash => (0.6 * e, 1.7 * e),
            Label::Intra => (0.4 * e, 2.0 * e),
            Label::Char => (0.6 * e, 1.9 * e),
            // Longer word gaps (a pause) are handled by a running maximum, see `viterbi`.
            Label::Word => (0.7 * e, e),
            Label::Lead => (0.0, 0.0),
        };
        ((lo.floor() as usize).max(1), (hi.ceil() as usize).max(2))
    }

    fn prior(l: Label) -> f64 {
        match l {
            Label::Dot => (0.55f64).ln(),
            Label::Dash => (0.45f64).ln(),
            Label::Intra => (0.55f64).ln(),
            Label::Char => (0.35f64).ln(),
            Label::Word => (0.10f64).ln(),
            Label::Lead => 0.0,
        }
    }

    fn logdur(&self, t: &Timing, l: Label, d: usize, open: bool) -> f64 {
        let e = Self::exp_dur(t, l);
        let s = if matches!(l, Label::Dot | Label::Dash) { self.cfg.mark_sigma } else { self.cfg.gap_sigma };
        let d = d.max(1) as f64;
        let mut z = (d / e).ln() / s;
        // A word gap can be as long as it likes. An open segment may still grow.
        if (l == Label::Word && d > e) || (open && d < e) {
            z = 0.0;
        }
        let pen = if matches!(l, Label::Dot | Label::Dash) { self.cfg.mark_penalty } else { 0.0 };
        -0.5 * z * z + Self::prior(l) - pen
    }

    // ---------- Viterbi ----------

    fn viterbi(&self, buf: &[f32], t: &Timing) -> Option<Path> {
        let n = buf.len();
        if n < 2 {
            return None;
        }
        let k = self.cfg.evidence_scale * 0.5;
        let mut cum = vec![0f64; n + 1];
        for i in 0..n {
            cum[i + 1] = cum[i] + buf[i] as f64 * k;
        }
        let mut m = vec![NEG; n + 1];
        let mut s = vec![NEG; n + 1];
        let mut mb = vec![Back::default(); n + 1];
        let mut sb = vec![Back::default(); n + 1];
        // Running maximum of m[a] + cum[a]: a gap longer than a word gap costs nothing extra,
        // so the best start of such a pause is simply the best earlier mark end. This keeps
        // pauses of any length cheap to search.
        let mut pref = vec![NEG; n + 1];
        let mut pref_at = vec![0usize; n + 1];
        let word_hi = Self::range(t, Label::Word).1;
        let word_prior = Self::prior(Label::Word);
        s[0] = 0.0;
        let marks = [Label::Dot, Label::Dash];
        let gaps = [Label::Intra, Label::Char, Label::Word];
        let ranges_m: Vec<(usize, usize)> = marks.iter().map(|l| Self::range(t, *l)).collect();
        let ranges_g: Vec<(usize, usize)> = gaps.iter().map(|l| Self::range(t, *l)).collect();
        for e in 1..=n {
            // Free lead: any amount of silence before the first mark.
            let lead = -cum[e];
            if lead > s[e] {
                s[e] = lead;
                sb[e] = Back { from: 0, label: 5 };
            }
            for (li, l) in marks.iter().enumerate() {
                let (lo, hi) = ranges_m[li];
                for d in lo..=hi.min(e) {
                    let a = e - d;
                    if s[a] == NEG {
                        continue;
                    }
                    let sc = s[a] + (cum[e] - cum[a]) + self.logdur(t, *l, d, false);
                    if sc > m[e] {
                        m[e] = sc;
                        mb[e] = Back { from: a, label: li as u8 };
                    }
                }
            }
            if e >= word_hi + 1 {
                let lim = e - word_hi - 1;
                if pref[lim] > NEG {
                    let sc = pref[lim] - cum[e] + word_prior;
                    if sc > s[e] {
                        s[e] = sc;
                        sb[e] = Back { from: pref_at[lim], label: 4 };
                    }
                }
            }
            for (li, l) in gaps.iter().enumerate() {
                let (lo, hi) = ranges_g[li];
                for d in lo..=hi.min(e) {
                    let a = e - d;
                    if m[a] == NEG {
                        continue;
                    }
                    let sc = m[a] - (cum[e] - cum[a]) + self.logdur(t, *l, d, false);
                    if sc > s[e] {
                        s[e] = sc;
                        sb[e] = Back { from: a, label: (li + 2) as u8 };
                    }
                }
            }
            let here = if m[e] > NEG { m[e] + cum[e] } else { NEG };
            if here > pref[e - 1] {
                pref[e] = here;
                pref_at[e] = e;
            } else {
                pref[e] = pref[e - 1];
                pref_at[e] = pref_at[e - 1];
            }
        }
        // End: either a complete segment ending exactly at n, or an open one still running.
        #[derive(Clone, Copy)]
        struct End {
            score: f64,
            is_mark: bool,
            open_from: Option<(usize, Label)>,
        }
        let mut best = End { score: NEG, is_mark: false, open_from: None };
        if m[n] > best.score {
            best = End { score: m[n], is_mark: true, open_from: None };
        }
        if s[n] > best.score {
            best = End { score: s[n], is_mark: false, open_from: None };
        }
        for l in marks.iter() {
            let (_, hi) = Self::range(t, *l);
            for d in 1..=hi.min(n) {
                let a = n - d;
                if s[a] == NEG {
                    continue;
                }
                let sc = s[a] + (cum[n] - cum[a]) + self.logdur(t, *l, d, true);
                if sc > best.score {
                    best = End { score: sc, is_mark: true, open_from: Some((a, *l)) };
                }
            }
        }
        // An open word gap or pause: any length from the word gap minimum on, all equally good.
        let word_lo = Self::range(t, Label::Word).0;
        if n >= word_lo && pref[n - word_lo] > NEG {
            let sc = pref[n - word_lo] - cum[n] + word_prior;
            if sc > best.score {
                best = End { score: sc, is_mark: false, open_from: Some((pref_at[n - word_lo], Label::Word)) };
            }
        }
        for l in [Label::Intra, Label::Char].iter() {
            let (_, hi) = Self::range(t, *l);
            for d in 1..=hi.min(n) {
                let a = n - d;
                if m[a] == NEG {
                    continue;
                }
                let sc = m[a] - (cum[n] - cum[a]) + self.logdur(t, *l, d, true);
                if sc > best.score {
                    best = End { score: sc, is_mark: false, open_from: Some((a, *l)) };
                }
            }
        }
        if best.score == NEG {
            return None;
        }
        let mut segs: Vec<Seg> = Vec::new();
        let (mut pos, mut in_mark);
        if let Some((a, l)) = best.open_from {
            segs.push(Seg { label: l, start: a, end: n, open: true });
            pos = a;
            in_mark = !best.is_mark; // the segment before an open mark is a space, and vice versa
        } else {
            pos = n;
            in_mark = best.is_mark;
        }
        while pos > 0 {
            let b = if in_mark { mb[pos] } else { sb[pos] };
            let l = lab(b.label);
            if in_mark && !matches!(l, Label::Dot | Label::Dash) || !in_mark && matches!(l, Label::Dot | Label::Dash) {
                return None;
            }
            segs.push(Seg { label: l, start: b.from, end: pos, open: false });
            pos = b.from;
            if l == Label::Lead {
                break;
            }
            in_mark = !in_mark;
        }
        segs.reverse();
        // A gap that runs up to the end of the buffer is still running, complete or not.
        if let Some(last) = segs.last_mut() {
            if last.end == n && matches!(last.label, Label::Intra | Label::Char | Label::Word) {
                last.open = true;
            }
        }
        Some(Path { score: best.score, segs })
    }

    fn timing(&self, u: f64) -> Timing {
        Timing { u, dash: self.dash, gap: self.gap, intra: self.intra }
    }

    fn candidate_units(&self, around: Option<f64>) -> Vec<f64> {
        if let Some(w) = self.cfg.fixed_wpm {
            return vec![240.0 / w];
        }
        match around {
            Some(u) => {
                // Never outside the speed range the operator set.
                let (lo, hi) = (240.0 / self.cfg.wpm_max, 240.0 / self.cfg.wpm_min);
                let mut v: Vec<f64> = [0.84, 0.92, 1.0, 1.09, 1.19].iter().map(|f| (u * f).clamp(lo, hi)).collect();
                v.dedup();
                v
            }
            None => {
                let (lo, hi) = (self.cfg.wpm_min, self.cfg.wpm_max);
                let steps = 34;
                (0..=steps).map(|i| 240.0 / (lo * (hi / lo).powf(i as f64 / steps as f64))).collect()
            }
        }
    }

    fn best_path(&self, buf: &[f32], cands: &[f64]) -> Option<(f64, Path)> {
        let mut best: Option<(f64, Path)> = None;
        for &u in cands {
            if u < 2.0 {
                continue;
            }
            if let Some(mut p) = self.viterbi(buf, &self.timing(u)) {
                p.score += Self::plausibility(&p.segs);
                if best.as_ref().map_or(true, |b| p.score > b.1.score) {
                    best = Some((u, p));
                }
            }
        }
        best
    }

    /// Morse text is not made of E and T only. A speed at which nearly every mark stands
    /// alone as a character is the wrong speed (too fast by half: the gaps inside characters
    /// are then read as gaps between them). About a fifth of real text is E or T; beyond
    /// 45 percent every further lone mark costs one nat.
    fn plausibility(segs: &[Seg]) -> f64 {
        let (mut groups, mut singles, mut in_group) = (0usize, 0usize, 0usize);
        for sg in segs {
            match sg.label {
                Label::Dot | Label::Dash => in_group += 1,
                Label::Char | Label::Word => {
                    if in_group > 0 {
                        groups += 1;
                        singles += usize::from(in_group == 1);
                    }
                    in_group = 0;
                }
                _ => {}
            }
        }
        if groups < 6 {
            return 0.0;
        }
        -(singles as f64 - 0.45 * groups as f64).max(0.0)
    }

    // ---------- characters ----------

    /// Posterior over the characters the same mark durations could spell.
    fn char_posterior(&self, durs: &[usize], t: &Timing) -> Vec<(char, f32)> {
        let k = durs.len();
        if k == 0 || k >= self.patterns.len() {
            return Vec::new();
        }
        let mut scored: Vec<(char, f64)> = Vec::new();
        for (c, bits) in &self.patterns[k] {
            let mut sc = 0.0;
            for (d, dash) in durs.iter().zip(bits) {
                sc += self.logdur(t, if *dash { Label::Dash } else { Label::Dot }, *d, false);
            }
            scored.push((*c, sc));
        }
        if scored.is_empty() {
            return Vec::new();
        }
        let mx = scored.iter().map(|x| x.1).fold(NEG, f64::max);
        let z: f64 = scored.iter().map(|x| (x.1 - mx).exp()).sum();
        let mut out: Vec<(char, f32)> = scored.iter().map(|x| (x.0, ((x.1 - mx).exp() / z) as f32)).collect();
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        out
    }

    fn group_to_char(&self, durs: &[usize], t: &Timing) -> TentativeChar {
        let post = self.char_posterior(durs, t);
        match post.first() {
            Some(&(c, p)) => TentativeChar { best: c, confidence: p, alts: post.iter().skip(1).filter(|x| x.1 > 0.02).take(3).cloned().collect() },
            None => TentativeChar { best: '?', confidence: 0.0, alts: Vec::new() },
        }
    }

    fn emit(&mut self, c: char, conf: f32) {
        self.emit_with(c, conf, Vec::new(), 99.0, 0.0, Vec::new());
    }

    fn emit_with(&mut self, c: char, conf: f32, alts: Vec<(char, f32)>, strength: f32, gap: f32, intra: Vec<f32>) {
        if c == ' ' {
            if !self.emitted_any || self.space_done {
                return;
            }
            self.space_done = true;
        } else {
            self.space_done = false;
        }
        self.emitted_any = true;
        self.committed.push(c);
        self.conf.push(conf);
        self.alts.push(alts);
        self.strength.push(strength);
        self.gaps.push(gap);
        self.intras.push(intra);
    }

    // ---------- the streaming step ----------

    fn step(&mut self) {
        let n = self.llr.len();
        if n < 40 {
            return;
        }
        let buf = std::mem::take(&mut self.llr);
        let result = self.step_inner(&buf);
        match result {
            Some(cut) => {
                self.llr = buf[cut.min(buf.len())..].to_vec();
            }
            None => self.llr = buf,
        }
        // An unlocked buffer must not grow without limit (a lot of noise before a signal).
        // Locked or not: a buffer that cannot be committed (a long carrier, garbage) is cut,
        // so the search stays fast.
        let cap = if self.locked { 3200 } else { 2400 };
        if self.llr.len() > cap {
            let drop = self.llr.len() - cap / 2;
            self.llr.drain(..drop);
        }
    }

    /// Runs the search and commits what is safe. Returns how many frames can be dropped.
    fn step_inner(&mut self, buf: &[f32]) -> Option<usize> {
        let window = &buf[buf.len().saturating_sub(1600)..];
        // The wide speed search is expensive: twice a second while looking for a signal,
        // every four seconds once locked (to notice a new station at another speed).
        let offset = buf.len() - window.len();
        let global_every = if self.locked { 4 * FRAME_RATE as usize } else { FRAME_RATE as usize / 2 };
        let global_due = self.since_global >= global_every;
        if self.u.is_none() && !global_due {
            return None;
        }
        let local = if self.u.is_some() { self.candidate_units(self.u) } else { Vec::new() };
        let mut chosen = self.best_path(window, &local);
        let mut switched = false;
        if global_due {
            self.since_global = 0;
            let global = self.candidate_units(None);
            let gwin = &window[window.len().saturating_sub(1000)..];
            let gres = if gwin.len() == window.len() { self.best_path(window, &global) } else {
                // Search the speed on the recent part, then score that speed on the whole window.
                self.best_path(gwin, &global).and_then(|(u, _)| self.best_path(window, &[u]))
            };
            if let Some(g) = gres {
                self.since_global = 0;
                let better = match &chosen {
                    None => true,
                    Some(l) => g.1.score > l.1.score + 12.0,
                };
                if better {
                    chosen = Some(g);
                    switched = true;
                }
            }
        }
        let (mut u, mut path) = chosen?;
        let marks = path.segs.iter().filter(|s| matches!(s.label, Label::Dot | Label::Dash) && !s.open).count();
        if !self.locked {
            // With only a few marks the speed is a guess, and the first characters would be
            // wrong. Wait for more data (a flush accepts less).
            if marks < if self.force_lock { 2 } else { 8 } {
                return None;
            }
            self.locked = true;
            if self.cfg.learn_style && marks >= 8 {
                // A new sender: take his style (dash length, weighting, spacing) from these first
                // marks at once and read them again with it, before anything is shown. Without
                // this the start of every transmission was read with the textbook style.
                for _ in 0..2 {
                    let (d, a, i, g) = Self::style_of(&path.segs, u, path.segs.len());
                    self.learn(&d, &a, &i, &g, 0.7);
                    match self.best_path(window, &self.candidate_units(Some(u))) {
                        Some(b) => (u, path) = b,
                        None => break,
                    }
                }
                self.learned = 0;
            }
        }
        // The speed only follows real marks. In a silence (or with one lonely blip) every
        // search ends a little faster than the last, and the estimate ran away to 100 WPM
        // during a pause, after which the next transmission came out as single letters.
        self.u = Some(match self.u {
            Some(old) if marks < 2 => old,
            Some(old) if !switched => (old.ln() * 0.5 + u.ln() * 0.5).exp(),
            _ => u,
        });
        let t = self.timing(self.u.unwrap());
        let segs = &path.segs;

        // How far may we commit? Up to the end of the last complete character gap that lies at
        // least four units back, or up to a long silence at the end.
        let n = window.len();
        let lag = (4.0 * t.u) as usize;
        let mut cut_seg: Option<usize> = None; // index of the gap segment where we cut
        for (i, sg) in segs.iter().enumerate() {
            if matches!(sg.label, Label::Char | Label::Word) && !sg.open && sg.end + lag <= n {
                cut_seg = Some(i);
            }
        }
        let mut tail_cut = false;
        if let Some(last) = segs.last() {
            if last.open && matches!(last.label, Label::Char | Label::Word) {
                tail_cut = true;
            }
        }
        // Silence only (no marks at all in the path)?
        let has_marks = segs.iter().any(|s| matches!(s.label, Label::Dot | Label::Dash));
        let last_end_is_silence = segs.last().map_or(false, |s| matches!(s.label, Label::Lead | Label::Intra | Label::Char | Label::Word));
        if !has_marks && last_end_is_silence {
            let run = self.carry + n;
            let word_thr = (5.2 * t.u * t.gap) as usize;
            if self.emitted_any && !self.space_done && run >= word_thr {
                self.emit(' ', 1.0);
            }
            // Keep memory of the silence but do not keep the frames.
            self.carry = run.min(100_000);
            self.trail = self.carry.max(self.gap_carry);
            self.refresh_tentative(&[], &t);
            return Some(offset + n);
        }

        let upto_idx = if tail_cut {
            segs.len() - 1
        } else {
            match cut_seg {
                Some(i) => i,
                None => {
                    self.trail = 0;
                    return None;
                }
            }
        };
        // Silence before the next character to come out, in frames.
        let mut gap_before = self.carry.max(self.gap_carry) + segs.first().filter(|s| s.label == Label::Lead).map_or(0, |s| s.end - s.start);
        // Emit the characters of segs[0..=upto_idx].
        let mut group: Vec<usize> = Vec::new();
        let mut group_intra: Vec<f32> = Vec::new();
        let mut group_strength = f32::MAX;
        let mut first_mark_start: Option<usize> = None;
        let lead_len = segs.first().filter(|s| s.label == Label::Lead).map_or(0, |s| s.end - s.start);
        let mut emitted_for_first = false;
        for (i, sg) in segs.iter().enumerate().take(upto_idx + 1) {
            match sg.label {
                Label::Dot | Label::Dash => {
                    if first_mark_start.is_none() {
                        first_mark_start = Some(sg.start);
                    }
                    group.push(sg.end - sg.start);
                    let fr = &window[sg.start..sg.end];
                    let mean = fr.iter().sum::<f32>() / fr.len().max(1) as f32;
                    group_strength = group_strength.min(mean);
                }
                Label::Char | Label::Word => {
                    if !group.is_empty() {
                        if !emitted_for_first {
                            emitted_for_first = true;
                            let silence_before = self.carry + lead_len;
                            let word_thr = (5.2 * t.u * t.gap) as usize;
                            if silence_before >= word_thr {
                                self.emit(' ', 1.0);
                            }
                        }
                        let tc = self.group_to_char(&group, &t);
                        self.emit_with(tc.best, tc.confidence, tc.alts.clone(), group_strength, (gap_before as f64 / t.u).min(99.0) as f32, std::mem::take(&mut group_intra));
                        group.clear();
                        group_strength = f32::MAX;
                    }
                    if sg.label == Label::Word {
                        self.emit(' ', 1.0);
                    }
                    gap_before = sg.end - sg.start;
                    let _ = i;
                }
                Label::Intra => group_intra.push(((sg.end - sg.start) as f64 / t.u) as f32),
                _ => {}
            }
        }
        if self.cfg.learn_style {
            let (d, a, i, g) = Self::style_of(segs, t.u, upto_idx + 1);
            // Fast while the sender is new, slowly once his style is known.
            let rate = if self.learned < 40 { 0.3 } else { 0.12 };
            self.learn(&d, &a, &i, &g, rate);
        }

        let cut_frames = if tail_cut {
            let last = segs.last().unwrap();
            self.carry = last.end - last.start;
            self.gap_carry = 0;
            self.trail = self.carry;
            n
        } else {
            self.carry = 0;
            self.gap_carry = segs[upto_idx].end - segs[upto_idx].start;
            self.trail = 0;
            segs[upto_idx].end
        };
        // The not yet final rest, shown as tentative text.
        let rest: Vec<Seg> = if tail_cut { Vec::new() } else { segs[upto_idx + 1..].to_vec() };
        self.refresh_tentative_segs(&rest, &t);
        Some(offset + cut_frames)
    }

    fn refresh_tentative(&mut self, _s: &[Seg], _t: &Timing) {
        self.tentative.clear();
    }

    fn refresh_tentative_segs(&mut self, segs: &[Seg], t: &Timing) {
        let mut out: Vec<TentativeChar> = Vec::new();
        let mut group: Vec<usize> = Vec::new();
        for sg in segs {
            match sg.label {
                Label::Dot | Label::Dash => group.push(sg.end - sg.start),
                Label::Char | Label::Word => {
                    if !group.is_empty() {
                        out.push(self.group_to_char(&group, t));
                        group.clear();
                    }
                }
                _ => {}
            }
        }
        if !group.is_empty() {
            let mut tc = self.group_to_char(&group, t);
            tc.confidence = tc.confidence.min(0.5);
            out.push(tc);
        }
        self.tentative = out;
    }

    /// The measured style in the first `upto` segments of a path: dot and dash lengths in
    /// frames, gaps inside characters and gaps between characters or words in units.
    fn style_of(segs: &[Seg], u: f64, upto: usize) -> (Vec<usize>, Vec<usize>, Vec<f64>, Vec<f64>) {
        let (mut dots, mut dashes, mut intra, mut inter) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for sg in segs.iter().take(upto).filter(|s| !s.open) {
            let d = sg.end - sg.start;
            match sg.label {
                Label::Dot => dots.push(d),
                Label::Dash => dashes.push(d),
                Label::Intra => intra.push(d as f64 / u),
                Label::Char | Label::Word => inter.push(d as f64 / u),
                Label::Lead => {}
            }
        }
        (dots, dashes, intra, inter)
    }

    /// The sender changed from wide to tight spacing (he spelled a name slowly and now keys on
    /// at his usual pace). His new gaps between characters are then shorter than the boundary
    /// learned from the wide part, so they are read as gaps inside characters and the
    /// characters run together. It shows in those gaps themselves: they fall in two groups,
    /// the real ones around one unit and a second group well above. When that second group
    /// is clearly shorter than the gap between characters believed so far, it is the new gap
    /// between characters, and it is taken over at once instead of followed slowly.
    fn tightened(&mut self) -> bool {
        let mut v: Vec<f64> = self.intra_hist.iter().cloned().collect();
        // Only a sender whose spacing was learned as clearly wide can have tightened it.
        if v.len() < 6 || self.gap < 1.3 {
            return false;
        }
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // The widest step with at least three gaps below it and two above.
        let Some(cut) = (3..=v.len() - 2).max_by(|a, b| (v[*a] / v[*a - 1]).partial_cmp(&(v[*b] / v[*b - 1])).unwrap()) else { return false };
        let (low, high) = (v[cut / 2], v[cut + (v.len() - cut) / 2]);
        if v[cut] / v[cut - 1] < 1.5 || high < 1.8 * low || 3.0 * self.gap < 1.5 * high {
            return false;
        }
        self.spacing_changed = true;
        self.gap = (high / 3.0).clamp(0.75, 3.5);
        self.intra = low.clamp(0.8, 2.2);
        self.gap_hist.clear();
        self.gap_hist.extend(v[cut..].iter().cloned());
        self.intra_hist.retain(|g| *g < v[cut]);
        true
    }

    /// Learns the sender's own style: dash length, weighting and spacing. `rate` is the share
    /// the new measurement gets.
    fn learn(&mut self, dots: &[usize], dashes: &[usize], intra: &[f64], inter: &[f64], rate: f64) {
        let median = |v: &[f64]| {
            let mut s = v.to_vec();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s[s.len() / 2]
        };
        self.learned += dots.len() + dashes.len();
        if !dots.is_empty() && !dashes.is_empty() {
            let md = dots.iter().sum::<usize>() as f64 / dots.len() as f64;
            let ma = dashes.iter().sum::<usize>() as f64 / dashes.len() as f64;
            let ratio = (ma / md.max(1.0)).clamp(2.2, 4.6);
            self.dash = (1.0 - rate) * self.dash + rate * ratio;
        }
        for g in intra {
            self.intra_hist.push_back(*g);
        }
        while self.intra_hist.len() > 16 {
            self.intra_hist.pop_front();
        }
        if self.tightened() {
            return;
        }
        if intra.len() >= 3 {
            // Weighting: the gap inside a character against the dot.
            let m = median(intra).clamp(0.8, 2.2);
            self.intra = (1.0 - rate) * self.intra + rate * m;
        }
        for g in inter {
            self.gap_hist.push_back(g.min(20.0));
        }
        while self.gap_hist.len() > 24 {
            self.gap_hist.pop_front();
        }
        if !inter.is_empty() && self.gap_hist.len() >= 4 {
            // The usual gap between characters is the lower of two groups (word gaps are the
            // upper one). With one group only, all of them are gaps between characters.
            let v: Vec<f64> = self.gap_hist.iter().map(|g| g.ln()).collect();
            let (mut lo, mut hi) = (v.iter().cloned().fold(f64::MAX, f64::min), v.iter().cloned().fold(f64::MIN, f64::max));
            let mut center = median(&self.gap_hist.iter().cloned().collect::<Vec<f64>>());
            if hi - lo > (1.6f64).ln() {
                for _ in 0..12 {
                    let cut = (lo + hi) / 2.0;
                    let a: Vec<f64> = v.iter().cloned().filter(|x| *x <= cut).collect();
                    let b: Vec<f64> = v.iter().cloned().filter(|x| *x > cut).collect();
                    if a.is_empty() || b.is_empty() {
                        break;
                    }
                    lo = a.iter().sum::<f64>() / a.len() as f64;
                    hi = b.iter().sum::<f64>() / b.len() as f64;
                }
                if hi - lo > (1.5f64).ln() {
                    center = lo.exp();
                }
            }
            let target = (center / 3.0).clamp(0.75, 3.5);
            self.gap = (1.0 - rate) * self.gap + rate * target;
        }
    }
}
