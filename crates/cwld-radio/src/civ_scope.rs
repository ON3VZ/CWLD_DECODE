//! The Icom spectrum scope over CI-V (command `27 00`): puts the pieces of one sweep together.
//!
//! Source: IC-7300 MK II CI-V Reference Guide, command 27. Over USB one sweep arrives in 11
//! pieces ("divisions"). Piece 1 holds only the description (mode, frequencies, out of range).
//! Pieces 2 to 11 hold the levels: 50 points each, the last one 25, so 475 points in all.
//! Each level runs from 0 to 160 (`00` to `A0`). Over LAN everything comes in one piece.
//!
//! **[TEST]**: in centre mode the guide gives "centre frequency and span". We read the span
//! as the distance from the centre to each edge (the radio shows "±25k"), to be checked on
//! the radio.

use crate::civ::bcd_to_freq;
use cwld_types::ScopeSweep;

pub const SCOPE: u8 = 0x27;
pub const SCOPE_DATA: u8 = 0x00;
/// Highest level the radio sends.
pub const MAX_LEVEL: u8 = 160;

/// Body of "scope on" (`27 10 01`) and "send the scope data to the PC" (`27 11 01`).
pub fn scope_on(on: bool) -> [u8; 3] {
    [SCOPE, 0x10, u8::from(on)]
}
pub fn scope_output(on: bool) -> [u8; 3] {
    [SCOPE, 0x11, u8::from(on)]
}

fn bcd1(b: u8) -> Option<u8> {
    let (hi, lo) = (b >> 4, b & 0x0F);
    if hi > 9 || lo > 9 {
        None
    } else {
        Some(hi * 10 + lo)
    }
}

#[derive(Default)]
pub struct ScopeAssembler {
    head: Option<Head>,
    data: Vec<u8>,
    next: u8,
}

#[derive(Clone, Copy)]
struct Head {
    lo: u64,
    hi: u64,
    centered: bool,
    out_of_range: bool,
    max: u8,
}

impl ScopeAssembler {
    /// Takes the body of one frame (`27 00 ...`). Returns a sweep when its last piece arrived.
    /// Pieces out of order or a damaged piece drop the sweep; the next one starts clean.
    pub fn push(&mut self, body: &[u8]) -> Option<ScopeSweep> {
        if body.len() < 5 || body[0] != SCOPE || body[1] != SCOPE_DATA {
            return None;
        }
        let d = &body[2..];
        let (Some(order), Some(max)) = (bcd1(d[1]), bcd1(d[2])) else {
            self.head = None;
            return None;
        };
        if order == 0 || max == 0 || order > max {
            self.head = None;
            return None;
        }
        if order == 1 {
            self.head = None;
            self.data.clear();
            if d.len() < 15 {
                return None;
            }
            let mode = d[3];
            let (Some(f1), Some(f2)) = (bcd_to_freq(&d[4..9]), bcd_to_freq(&d[9..14])) else { return None };
            let centered = mode == 0 || mode == 2;
            let (lo, hi) = if centered { (f1.saturating_sub(f2), f1 + f2) } else { (f1, f2) };
            if hi <= lo {
                return None;
            }
            let head = Head { lo, hi, centered, out_of_range: d[14] != 0, max };
            self.head = Some(head);
            self.next = 2;
            if max == 1 {
                // LAN: description and levels in one piece.
                self.data.extend(d[15..].iter().map(|v| (*v).min(MAX_LEVEL)));
                return self.finish();
            }
            if head.out_of_range {
                // The levels are left out.
                return self.finish();
            }
            return None;
        }
        let head = self.head?;
        if order != self.next || max != head.max {
            self.head = None;
            return None;
        }
        self.data.extend(d[3..].iter().map(|v| (*v).min(MAX_LEVEL)));
        self.next += 1;
        if order == max {
            return self.finish();
        }
        None
    }

    fn finish(&mut self) -> Option<ScopeSweep> {
        let head = self.head.take()?;
        let data = std::mem::take(&mut self.data);
        if !head.out_of_range && data.is_empty() {
            return None;
        }
        Some(ScopeSweep { lo_hz: head.lo, hi_hz: head.hi, centered: head.centered, out_of_range: head.out_of_range, data })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::civ::freq_to_bcd;

    fn bcd(v: u8) -> u8 {
        ((v / 10) << 4) | (v % 10)
    }

    /// The 11 frame bodies of one USB sweep, as the radio sends them. Test helper.
    pub fn usb_sweep(mode: u8, f1: u64, f2: u64, level: impl Fn(usize) -> u8) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let mut first = vec![SCOPE, SCOPE_DATA, 0x00, bcd(1), bcd(11), mode];
        first.extend_from_slice(&freq_to_bcd(f1));
        first.extend_from_slice(&freq_to_bcd(f2));
        first.push(0x00);
        assert_eq!(first.len() - 2, 15);
        out.push(first);
        let mut i = 0;
        for order in 2..=11u8 {
            let n = if order == 11 { 25 } else { 50 };
            let mut b = vec![SCOPE, SCOPE_DATA, 0x00, bcd(order), bcd(11)];
            for _ in 0..n {
                b.push(level(i));
                i += 1;
            }
            out.push(b);
        }
        out
    }

    #[test]
    fn eleven_pieces_make_one_sweep_of_475_points() {
        let mut a = ScopeAssembler::default();
        let mut got = None;
        for b in usb_sweep(0x00, 7_030_000, 25_000, |i| (i % 161) as u8) {
            if let Some(s) = a.push(&b) {
                got = Some(s);
            }
        }
        let s = got.unwrap();
        assert_eq!((s.lo_hz, s.hi_hz, s.centered, s.out_of_range), (7_005_000, 7_055_000, true, false));
        assert_eq!(s.data.len(), 475);
        assert_eq!(s.data[160], 160);
        assert_eq!(s.data[474], (474 % 161) as u8);
    }

    #[test]
    fn fixed_mode_gives_the_edges() {
        let mut a = ScopeAssembler::default();
        let s = usb_sweep(0x01, 7_000_000, 7_040_000, |_| 10).iter().find_map(|b| a.push(b)).unwrap();
        assert_eq!((s.lo_hz, s.hi_hz, s.centered), (7_000_000, 7_040_000, false));
    }

    #[test]
    fn a_missing_piece_drops_the_sweep_and_the_next_one_is_fine() {
        let mut a = ScopeAssembler::default();
        let mut pieces = usb_sweep(0x00, 7_030_000, 5_000, |_| 20);
        pieces.remove(5);
        assert!(pieces.iter().all(|b| a.push(b).is_none()));
        let s = usb_sweep(0x00, 7_030_000, 5_000, |_| 20).iter().find_map(|b| a.push(b));
        assert_eq!(s.unwrap().data.len(), 475);
    }

    #[test]
    fn out_of_range_comes_without_levels() {
        let mut a = ScopeAssembler::default();
        let mut first = usb_sweep(0x00, 7_030_000, 5_000, |_| 0).remove(0);
        *first.last_mut().unwrap() = 0x01;
        let s = a.push(&first).unwrap();
        assert!(s.out_of_range && s.data.is_empty());
    }

    #[test]
    fn levels_above_160_are_clipped_and_lan_comes_in_one_piece() {
        let mut a = ScopeAssembler::default();
        let mut b = vec![SCOPE, SCOPE_DATA, 0x00, bcd(1), bcd(1), 0x00];
        b.extend_from_slice(&freq_to_bcd(14_030_000));
        b.extend_from_slice(&freq_to_bcd(10_000));
        b.push(0);
        b.extend(std::iter::repeat(0xF0).take(475));
        let s = a.push(&b).unwrap();
        assert_eq!(s.data.len(), 475);
        assert!(s.data.iter().all(|v| *v == 160));
    }
}
