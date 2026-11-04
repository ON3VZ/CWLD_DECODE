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
}

impl Default for DecoderConfig {
    fn default() -> Self {
        DecoderConfig { wpm_min: 5.0, wpm_max: 50.0, evidence_scale: 0.1, mark_sigma: 0.28, gap_sigma: 0.30, tentative_below: 0.9, mark_penalty: 0.0 }
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
        up.tentative = self.tentative.clone();
        up.wpm = self.wpm().unwrap_or(0.0) as f32;
        up.locked = self.locked;
        up
    }

    // ---------- timing model ----------

    fn exp_dur(t: &Timing, l: Label) -> f64 {
        match l {
            Label::Dot | Label::Intra => t.u,
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
        Timing { u, dash: self.dash, gap: self.gap }
    }

    fn candidate_units(&self, around: Option<f64>) -> Vec<f64> {
        match around {
            Some(u) => [0.84, 0.92, 1.0, 1.09, 1.19].iter().map(|f| u * f).collect(),
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
            if let Some(p) = self.viterbi(buf, &self.timing(u)) {
                if best.as_ref().map_or(true, |b| p.score > b.1.score) {
                    best = Some((u, p));
                }
            }
        }
        best
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
        self.emit_with(c, conf, Vec::new(), 99.0);
    }

    fn emit_with(&mut self, c: char, conf: f32, alts: Vec<(char, f32)>, strength: f32) {
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
        let (u, path) = chosen?;
        let marks = path.segs.iter().filter(|s| matches!(s.label, Label::Dot | Label::Dash) && !s.open).count();
        if !self.locked {
            // With only a few marks the speed is a guess, and the first characters would be
            // wrong. Wait for more data (a flush accepts less).
            if marks < if self.force_lock { 2 } else { 8 } {
                return None;
            }
            self.locked = true;
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
            self.refresh_tentative(&[], &t);
            return Some(offset + n);
        }

        let upto_idx = if tail_cut { segs.len() - 1 } else { cut_seg.map(|i| i)? + 0 };
        // Emit the characters of segs[0..=upto_idx].
        let mut group: Vec<usize> = Vec::new();
        let mut group_strength = f32::MAX;
        let mut first_mark_start: Option<usize> = None;
        let mut dot_d: Vec<usize> = Vec::new();
        let mut dash_d: Vec<usize> = Vec::new();
        let mut gap_ratio: Vec<f64> = Vec::new();
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
                    if sg.label == Label::Dot {
                        dot_d.push(sg.end - sg.start);
                    } else {
                        dash_d.push(sg.end - sg.start);
                    }
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
                        self.emit_with(tc.best, tc.confidence, tc.alts.clone(), group_strength);
                        group.clear();
                        group_strength = f32::MAX;
                    }
                    if sg.label == Label::Word {
                        self.emit(' ', 1.0);
                    }
                    if sg.label == Label::Char && !sg.open {
                        gap_ratio.push((sg.end - sg.start) as f64 / (3.0 * t.u));
                    }
                    let _ = i;
                }
                _ => {}
            }
        }
        self.learn(&dot_d, &dash_d, &gap_ratio);

        let cut_frames = if tail_cut {
            let last = segs.last().unwrap();
            self.carry = last.end - last.start;
            n
        } else {
            self.carry = 0;
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

    /// Learns the sender's own style: dash length and spacing, slowly.
    fn learn(&mut self, dots: &[usize], dashes: &[usize], gaps: &[f64]) {
        let u = self.u.unwrap_or(12.0);
        if !dots.is_empty() && !dashes.is_empty() {
            let md = dots.iter().sum::<usize>() as f64 / dots.len() as f64;
            let ma = dashes.iter().sum::<usize>() as f64 / dashes.len() as f64;
            let ratio = (ma / md.max(1.0)).clamp(2.2, 4.2);
            self.dash = 0.9 * self.dash + 0.1 * ratio;
        }
        let _ = u;
        if !gaps.is_empty() {
            let mg = gaps.iter().sum::<f64>() / gaps.len() as f64;
            // Word gaps pull this up, so only trust it within sane bounds.
            self.gap = (0.92 * self.gap + 0.08 * mg).clamp(0.8, 2.0);
        }
    }
}
