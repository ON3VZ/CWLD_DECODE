//! The Icom CI-V protocol: frames, numbers and codes. No I/O here.
//!
//! Source: the IC-7300 MK II CI-V Reference Guide. Values marked **[TEST]** are not
//! confirmed by the guide and must be measured on the radio.

use cwld_types::RadioMode;

pub const PREAMBLE: u8 = 0xFE;
pub const END: u8 = 0xFD;
pub const OK: u8 = 0xFB;
pub const NG: u8 = 0xFA;
/// Address of the controller (the PC). `E0` is what Icom documents.
pub const CONTROLLER: u8 = 0xE0;
/// Default address of the IC-7300 MK II. The plain IC-7300 is `94`, so never mix their tables.
pub const DEFAULT_RADIO: u8 = 0xB6;
/// Broadcast address, used for the radio's unsolicited changes (CI-V Transceive).
pub const BROADCAST: u8 = 0x00;

/// Commands used by CWLD.
pub mod cmd {
    pub const FREQ_PUSH: u8 = 0x00;
    pub const MODE_PUSH: u8 = 0x01;
    pub const READ_FREQ: u8 = 0x03;
    pub const READ_MODE: u8 = 0x04;
    pub const SET_FREQ: u8 = 0x05;
    pub const SET_MODE: u8 = 0x06;
    pub const KEYER_SPEED: [u8; 2] = [0x14, 0x0C];
    pub const S_METER: [u8; 2] = [0x15, 0x02];
    pub const SEND_CW: u8 = 0x17;
    pub const TX_STATUS: [u8; 2] = [0x1C, 0x00];
    pub const SWR_METER: [u8; 2] = [0x15, 0x12];
    /// AF level (speaker volume), 0000 to 0255. Same as the IC-7300; to confirm on the MK II **[TEST]**.
    pub const AF_LEVEL: [u8; 2] = [0x14, 0x01];
    /// Break-in: 00 off, 01 semi, 02 full. Same as the IC-7300; to confirm on the MK II **[TEST]**.
    pub const BREAK_IN: [u8; 2] = [0x16, 0x47];
}

/// Builds one frame: `FE FE to from body FD`.
pub fn encode(to: u8, from: u8, body: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(body.len() + 5);
    v.extend_from_slice(&[PREAMBLE, PREAMBLE, to, from]);
    v.extend_from_slice(body);
    v.push(END);
    v
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub to: u8,
    pub from: u8,
    /// Command followed by its data.
    pub body: Vec<u8>,
}

/// Cuts a byte stream into frames. Tolerates noise between frames and frames split over reads.
#[derive(Default)]
pub struct Parser {
    buf: Vec<u8>,
}

impl Parser {
    pub fn push(&mut self, bytes: &[u8]) -> Vec<Frame> {
        self.buf.extend_from_slice(bytes);
        let mut frames = Vec::new();
        loop {
            // Find a preamble.
            let Some(start) = self.buf.windows(2).position(|w| w == [PREAMBLE, PREAMBLE]) else {
                // Keep a lone trailing FE, it may be the first half of a preamble.
                let keep = if self.buf.last() == Some(&PREAMBLE) { 1 } else { 0 };
                let cut = self.buf.len() - keep;
                self.buf.drain(..cut);
                break;
            };
            self.buf.drain(..start);
            // Extra FE bytes in a row: the real preamble is the last two.
            while self.buf.len() > 2 && self.buf[2] == PREAMBLE {
                self.buf.remove(0);
            }
            let Some(end) = self.buf.iter().skip(2).position(|&b| b == END).map(|p| p + 2) else { break };
            let raw: Vec<u8> = self.buf.drain(..=end).collect();
            // raw = FE FE to from body.. FD
            if raw.len() >= 5 {
                frames.push(Frame { to: raw[2], from: raw[3], body: raw[4..raw.len() - 1].to_vec() });
            }
        }
        // Never let garbage grow without bound.
        if self.buf.len() > 512 {
            self.buf.clear();
        }
        frames
    }
}

/// Frequency in Hz to 5 BCD bytes, least significant byte first.
pub fn freq_to_bcd(hz: u64) -> [u8; 5] {
    let mut out = [0u8; 5];
    let mut v = hz;
    for b in out.iter_mut() {
        let lo = (v % 10) as u8;
        v /= 10;
        let hi = (v % 10) as u8;
        v /= 10;
        *b = (hi << 4) | lo;
    }
    out
}

/// 5 BCD bytes (least significant first) to Hz. `None` if a nibble is not a digit.
pub fn bcd_to_freq(bytes: &[u8]) -> Option<u64> {
    if bytes.len() < 5 {
        return None;
    }
    let mut hz = 0u64;
    for b in bytes[..5].iter().rev() {
        let (hi, lo) = (b >> 4, b & 0x0F);
        if hi > 9 || lo > 9 {
            return None;
        }
        hz = hz * 100 + u64::from(hi) * 10 + u64::from(lo);
    }
    Some(hz)
}

/// A number 0 to 9999 as 2 BCD bytes, most significant first (`0255` is `02 55`).
pub fn bcd2(v: u16) -> [u8; 2] {
    let v = v.min(9999);
    [(((v / 1000) << 4) | ((v / 100) % 10)) as u8, ((((v / 10) % 10) << 4) | (v % 10)) as u8]
}

pub fn from_bcd2(b: &[u8]) -> Option<u16> {
    if b.len() < 2 {
        return None;
    }
    let digit = |n: u8| if n > 9 { None } else { Some(u16::from(n)) };
    Some(digit(b[0] >> 4)? * 1000 + digit(b[0] & 0xF)? * 100 + digit(b[1] >> 4)? * 10 + digit(b[1] & 0xF)?)
}

pub fn mode_code(m: RadioMode) -> Option<u8> {
    Some(match m {
        RadioMode::Lsb => 0x00,
        RadioMode::Usb => 0x01,
        RadioMode::Am => 0x02,
        RadioMode::Cw => 0x03,
        RadioMode::Fsk => 0x04,
        RadioMode::Fm => 0x05,
        RadioMode::CwReverse => 0x07,
        RadioMode::Other => return None,
    })
}

pub fn mode_from_code(c: u8) -> RadioMode {
    match c {
        0x00 => RadioMode::Lsb,
        0x01 => RadioMode::Usb,
        0x02 => RadioMode::Am,
        0x03 => RadioMode::Cw,
        0x04 | 0x08 => RadioMode::Fsk,
        0x05 => RadioMode::Fm,
        0x07 => RadioMode::CwReverse,
        _ => RadioMode::Other,
    }
}

/// Keyer speed range of the radio in WPM (`0000` is 6 WPM, `0255` is 48 WPM).
pub const MIN_WPM: u8 = 6;
pub const MAX_WPM: u8 = 48;

/// WPM to the radio's 0 to 255 scale. The line between the two ends is an assumption **[TEST]**.
pub fn wpm_to_raw(wpm: u8) -> u16 {
    let w = u16::from(wpm.clamp(MIN_WPM, MAX_WPM));
    ((w - u16::from(MIN_WPM)) as f64 * 255.0 / f64::from(MAX_WPM - MIN_WPM)).round() as u16
}

pub fn raw_to_wpm(raw: u16) -> u8 {
    (f64::from(MIN_WPM) + f64::from(raw.min(255)) * f64::from(MAX_WPM - MIN_WPM) / 255.0).round() as u8
}

/// Points of the SWR meter scale: raw value and the SWR it stands for.
/// 0 is 1.0, 48 is 1.5, 80 is 2.0, 120 is 3.0 **[VERIFIEER]** in the MK II guide, the end is a guess.
const SWR_SCALE: [(f32, f32); 5] = [(0.0, 1.0), (48.0, 1.5), (80.0, 2.0), (120.0, 3.0), (255.0, 10.0)];

/// Converts the raw 0 to 255 SWR meter value to an SWR, straight lines between the points.
pub fn swr_from_raw(raw: u16) -> f32 {
    let r = f32::from(raw.min(255));
    for w in SWR_SCALE.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if r <= x1 {
            return y0 + (y1 - y0) * (r - x0) / (x1 - x0);
        }
    }
    SWR_SCALE[SWR_SCALE.len() - 1].1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequency_bcd_matches_the_guide_example() {
        // 14.074 MHz: 00 40 07 14 00 (least significant byte first).
        assert_eq!(freq_to_bcd(14_074_000), [0x00, 0x40, 0x07, 0x14, 0x00]);
        assert_eq!(bcd_to_freq(&[0x00, 0x40, 0x07, 0x14, 0x00]), Some(14_074_000));
    }

    #[test]
    fn frequency_round_trips() {
        for hz in [30_000u64, 1_840_000, 7_025_400, 14_030_123, 28_074_000, 74_800_000] {
            assert_eq!(bcd_to_freq(&freq_to_bcd(hz)), Some(hz), "{hz}");
        }
    }

    #[test]
    fn bad_bcd_is_refused() {
        assert_eq!(bcd_to_freq(&[0x0A, 0, 0, 0, 0]), None);
        assert_eq!(bcd_to_freq(&[0, 0, 0]), None);
    }

    #[test]
    fn bcd2_is_most_significant_first() {
        assert_eq!(bcd2(255), [0x02, 0x55]);
        assert_eq!(bcd2(0), [0x00, 0x00]);
        assert_eq!(from_bcd2(&[0x02, 0x55]), Some(255));
        assert_eq!(from_bcd2(&[0x0F, 0x00]), None);
    }

    #[test]
    fn frame_encoding() {
        assert_eq!(encode(0xB6, 0xE0, &[0x03]), vec![0xFE, 0xFE, 0xB6, 0xE0, 0x03, 0xFD]);
    }

    #[test]
    fn parser_handles_noise_and_split_frames() {
        let mut p = Parser::default();
        let f = encode(0xE0, 0xB6, &[0x03, 0x00, 0x40, 0x07, 0x14, 0x00]);
        let (a, b) = f.split_at(4);
        assert!(p.push(&[0x00, 0x13]).is_empty());
        assert!(p.push(a).is_empty());
        let frames = p.push(b);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0], Frame { to: 0xE0, from: 0xB6, body: vec![0x03, 0x00, 0x40, 0x07, 0x14, 0x00] });
    }

    #[test]
    fn parser_gives_echo_and_answer_as_two_frames() {
        let mut p = Parser::default();
        let mut bytes = encode(0xB6, 0xE0, &[0x03]);
        bytes.extend(encode(0xE0, 0xB6, &[0xFB]));
        let frames = p.push(&bytes);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].from, 0xE0);
        assert_eq!(frames[1].body, vec![0xFB]);
    }

    #[test]
    fn parser_survives_a_burst_of_garbage() {
        let mut p = Parser::default();
        assert!(p.push(&vec![0x55; 2000]).is_empty());
        assert_eq!(p.push(&encode(0xE0, 0xB6, &[0xFB])).len(), 1);
    }

    #[test]
    fn swr_scale_points() {
        assert_eq!(swr_from_raw(0), 1.0);
        assert!((swr_from_raw(48) - 1.5).abs() < 1e-5);
        assert!((swr_from_raw(80) - 2.0).abs() < 1e-5);
        assert!((swr_from_raw(120) - 3.0).abs() < 1e-5);
        assert!(swr_from_raw(255) >= 3.0);
        assert!(swr_from_raw(64) > 1.5 && swr_from_raw(64) < 2.0);
    }

    #[test]
    fn modes() {
        assert_eq!(mode_code(RadioMode::Cw), Some(0x03));
        assert_eq!(mode_from_code(0x07), RadioMode::CwReverse);
        assert_eq!(mode_code(RadioMode::Other), None);
    }

    #[test]
    fn speed_scale_ends_match_the_guide() {
        assert_eq!(wpm_to_raw(6), 0);
        assert_eq!(wpm_to_raw(48), 255);
        assert_eq!(raw_to_wpm(0), 6);
        assert_eq!(raw_to_wpm(255), 48);
        for w in 6..=48u8 {
            assert_eq!(raw_to_wpm(wpm_to_raw(w)), w, "{w}");
        }
    }
}
