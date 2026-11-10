//! Driver for the QRP Labs QMX, which speaks a Kenwood TS-480 style CAT protocol in text.
//!
//! Source: the QMX CAT manual (firmware 1_04_004 and later). The manual is vague in places,
//! so everything marked **[TEST]** must be checked on a real QMX.

use crate::clock::Clock;
use crate::error::RigError;
use crate::keyer::{self, KeyerError, ProsignStyle};
use crate::rig::{Capabilities, CwTextMode, Rig, RigInfo};
use crate::transport::Transport;
use cwld_types::{RadioCommand, RadioMode, RadioStatus, RigEvent};
use std::collections::VecDeque;

/// Size of the `KY` text buffer of the radio.
pub const KY_BUFFER: usize = 80;
/// While the buffer is "less than 75% full", at most 60 characters are in it. A word is only
/// added when it surely fits next to that.
const SAFE_ROOM_UNDER_75: usize = KY_BUFFER - 60 - 1;
/// Size of the radio decoder text buffer (`TB`).
pub const TB_BUFFER: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ky {
    /// `KY0;` sending, buffer less than 75% full.
    Under75,
    /// `KY1;` sending, buffer more than 75% full.
    Over75,
    /// `KY2;` idle and empty.
    Empty,
}

/// Commands that CWLD must never send to a QMX, whatever calls the driver.
/// `PS0;` switches the radio off, `KS00;` puts it in straight-key mode.
pub fn is_forbidden(line: &str) -> bool {
    let l = line.trim().to_ascii_uppercase();
    l.starts_with("PS") || l == "KS00;" || l.starts_with("KS00") || l.starts_with("KS0;")
}

pub struct QmxRig<T: Transport> {
    t: T,
    clock: Box<dyn Clock>,
    rx_buf: String,
    status: RadioStatus,
    events: Vec<RigEvent>,
    words: VecDeque<String>,
    ky: Option<Ky>,
    /// Until this time the radio is sending our own text, so its decoder echo is not "paddle".
    own_text_until: u64,
    last_rx: Option<u64>,
    started: u64,
    asked_id: bool,
}

impl<T: Transport> QmxRig<T> {
    pub fn new(t: T, clock: Box<dyn Clock>) -> Self {
        let started = clock.now_ms();
        QmxRig {
            t,
            clock,
            rx_buf: String::new(),
            status: RadioStatus {
                connected: false,
                frequency_hz: 0,
                mode: RadioMode::Cw,
                filter: None,
                cw_wpm: 20,
                transmitting: false,
                s_meter: 0,
            },
            events: Vec::new(),
            words: VecDeque::new(),
            ky: None,
            own_text_until: 0,
            last_rx: None,
            started,
            asked_id: false,
        }
    }

    fn io_failed(&mut self, e: std::io::Error) -> RigError {
        if self.status.connected {
            self.status.connected = false;
            self.events.push(RigEvent::Connected(false));
        }
        RigError::Io(e.to_string())
    }

    /// The only way text leaves this driver. Forbidden commands stop here.
    fn send(&mut self, line: &str) -> Result<(), RigError> {
        if is_forbidden(line) {
            return Err(RigError::InvalidArgument("forbidden command".into()));
        }
        self.t.write_all(line.as_bytes()).map_err(|e| self.io_failed(e))
    }

    fn queue_event(&mut self) {
        self.events.push(RigEvent::Queue(self.words.iter().cloned().collect()));
    }

    /// Hands the next words to the radio while they surely fit in its buffer.
    ///
    /// Every word is sent with a trailing space, so appended words keep their word gap.
    /// The last word then has a harmless extra space **[TEST]**.
    fn pump(&mut self) -> Result<(), RigError> {
        let mut room = match self.ky {
            Some(Ky::Empty) => KY_BUFFER,
            Some(Ky::Under75) => SAFE_ROOM_UNDER_75,
            Some(Ky::Over75) => 0,
            // Unknown buffer state: never guess, wait for the first `KY;` answer.
            None => 0,
        };
        let mut sent_any = false;
        while let Some(word) = self.words.front().cloned() {
            let len = word.chars().count() + 1;
            if len > room {
                break;
            }
            let line = format!("KY {word} ;");
            self.send(&line)?;
            room -= len;
            let est = keyer::estimate_ms(&word, self.status.cw_wpm, ProsignStyle::Qmx) + 7 * (1200 / u64::from(self.status.cw_wpm.max(1)));
            let now = self.clock.now_ms();
            self.own_text_until = self.own_text_until.max(now) + est;
            self.words.pop_front();
            sent_any = true;
        }
        if sent_any {
            // We no longer know how full the buffer is: wait for the next `KY;` answer.
            self.ky = None;
            self.own_text_until += 400;
            self.queue_event();
        }
        Ok(())
    }

    /// Reads what the radio sent and handles every complete `...;` message.
    fn read_incoming(&mut self) -> Result<(), ()> {
        match self.t.read_available() {
            Ok(bytes) => {
                self.rx_buf.push_str(&String::from_utf8_lossy(&bytes));
                while let Some(end) = self.rx_buf.find(';') {
                    let msg: String = self.rx_buf.drain(..=end).collect();
                    let msg = msg.trim_end_matches(';').trim_start_matches(['\r', '\n']).to_string();
                    self.handle(&msg);
                }
                if self.rx_buf.len() > 512 {
                    self.rx_buf.clear();
                }
                Ok(())
            }
            Err(e) => {
                let _ = self.io_failed(e);
                Err(())
            }
        }
    }

    fn handle(&mut self, msg: &str) {
        let now = self.clock.now_ms();
        self.last_rx = Some(now);
        if !self.status.connected {
            self.status.connected = true;
            self.events.push(RigEvent::Connected(true));
        }
        if msg == "?" {
            self.events.push(RigEvent::Warning("qmx_rejected".into()));
            return;
        }
        if msg.len() < 2 || !msg.is_char_boundary(2) {
            return;
        }
        let (cmd, arg) = msg.split_at(2);
        match cmd {
            "ID" => {
                if arg != "020" {
                    self.events.push(RigEvent::Warning("not_a_qmx".into()));
                }
            }
            // `FA` answer, or the first 11 digits of an `IF` push. The rest of `IF` is not used,
            // mode and TX status come from their own commands **[TEST]**.
            "FA" | "IF" => {
                if let Some(d) = arg.get(..11).filter(|d| d.bytes().all(|b| b.is_ascii_digit())) {
                    if let Ok(hz) = d.parse::<u64>() {
                        if hz != self.status.frequency_hz {
                            self.status.frequency_hz = hz;
                            self.events.push(RigEvent::Frequency(hz));
                        }
                    }
                }
            }
            "MD" => {
                let m = match arg {
                    "1" => RadioMode::Lsb,
                    "2" => RadioMode::Usb,
                    "3" => RadioMode::Cw,
                    "5" => RadioMode::Am,
                    "6" | "9" => RadioMode::Fsk,
                    "7" => RadioMode::CwReverse,
                    _ => RadioMode::Other,
                };
                if m != self.status.mode {
                    self.status.mode = m;
                    self.events.push(RigEvent::Mode(m));
                }
            }
            "TQ" => {
                let tx = arg == "1";
                if tx != self.status.transmitting {
                    self.status.transmitting = tx;
                    self.events.push(RigEvent::Tx(tx));
                }
            }
            "KS" => {
                if let Ok(w) = arg.parse::<u8>() {
                    if w > 0 && w != self.status.cw_wpm {
                        self.status.cw_wpm = w;
                        self.events.push(RigEvent::CwSpeed(w));
                    }
                }
            }
            "KY" => {
                self.ky = match arg {
                    "0" => Some(Ky::Under75),
                    "1" => Some(Ky::Over75),
                    "2" => Some(Ky::Empty),
                    _ => self.ky,
                };
            }
            // `SWnnn;` is the SWR in hundredths (121 = 1.21) **[VERIFIEER]**. In receive the radio
            // answers a bare `SW;`: no value, so no reading. Values below 1.00 are nonsense.
            "SW" => {
                if let Ok(h) = arg.trim().parse::<u32>() {
                    if h >= 100 {
                        self.events.push(RigEvent::Swr(h as f32 / 100.0));
                    }
                }
            }
            "TB" => {
                // `TB` t nn text: t is the status, nn the number of characters.
                if arg.len() >= 3 {
                    let n: usize = arg[1..3].parse().unwrap_or(0);
                    let text: String = arg[3..].chars().take(n).collect();
                    // While the radio sends our own text, its decoder repeats it: not paddle.
                    if !text.is_empty() && now >= self.own_text_until && self.words.is_empty() {
                        self.events.push(RigEvent::PaddleText(text));
                    }
                }
            }
            _ => {}
        }
    }
}

impl<T: Transport> Rig for QmxRig<T> {
    fn info(&self) -> RigInfo {
        RigInfo { name: "QRP Labs QMX".to_string(), driver: "qmx" }
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities { cw_text: CwTextMode::Buffered { size: KY_BUFFER }, radio_decoder_text: true, dtr_keying: true, pushes_status: true, swr_meter: true, scope: false }
    }

    fn execute(&mut self, c: &RadioCommand) -> Result<(), RigError> {
        match c {
            RadioCommand::ReadStatus => self.send("FA;MD;TQ;KS;KY;"),
            RadioCommand::SetFrequency(hz) => self.send(&format!("FA{hz:011};")),
            RadioCommand::SetMode(m) => {
                let d = match m {
                    RadioMode::Lsb => 1,
                    RadioMode::Usb => 2,
                    RadioMode::Cw => 3,
                    RadioMode::Am => 5,
                    RadioMode::Fsk => 6,
                    RadioMode::CwReverse => 7,
                    RadioMode::Fm | RadioMode::Other => return Err(RigError::InvalidArgument("mode".into())),
                };
                self.send(&format!("MD{d};"))
            }
            RadioCommand::SetCwSpeed(w) => {
                if *w == 0 {
                    return Err(RigError::InvalidArgument("speed 0 is straight key on a QMX".into()));
                }
                self.send(&format!("KS{w:02};"))
            }
            RadioCommand::SendCw(text) => {
                let prepared = keyer::prepare(text, ProsignStyle::Qmx).map_err(|e| match e {
                    KeyerError::Empty => RigError::InvalidArgument("empty text".into()),
                    KeyerError::WordTooLong { .. } => RigError::InvalidArgument("word_too_long".into()),
                })?;
                if let Some(long) = prepared.words.iter().find(|w| w.chars().count() + 1 > KY_BUFFER) {
                    let _ = long;
                    return Err(RigError::InvalidArgument("word_too_long".into()));
                }
                let total: u64 = prepared.words.iter().map(|w| keyer::estimate_ms(w, self.status.cw_wpm, ProsignStyle::Qmx) + 7 * (1200 / u64::from(self.status.cw_wpm.max(1)))).sum();
                if total > keyer::MAX_MESSAGE_SECONDS * 1000 {
                    return Err(RigError::InvalidArgument("message_too_long".into()));
                }
                if !prepared.dropped.is_empty() {
                    self.events.push(RigEvent::Warning("dropped_chars".into()));
                }
                self.words.extend(prepared.words);
                self.queue_event();
                // Ask for the buffer state first, the answer decides how much fits.
                self.send("KY;")?;
                Ok(())
            }
            RadioCommand::StopCw => {
                self.words.clear();
                self.own_text_until = 0;
                self.queue_event();
                // No stop command is documented for the normal mode. In TS-480 compatibility mode
                // a message of 24 spaces stops sending **[TEST]**. Spaces never key the radio.
                self.send(&format!("KY {};", " ".repeat(24)))
            }
            RadioCommand::SetPtt(on) => self.send(if *on { "TQ1;" } else { "TQ0;" }),
            RadioCommand::KeyDown(on) => self.send(if *on { "KD1;" } else { "KD0;" }),
            RadioCommand::Raw(bytes) => {
                let line = String::from_utf8_lossy(bytes).to_string();
                self.send(&line)
            }
            // The QMX has no band scope.
            RadioCommand::SetScope(_) => Ok(()),
            // Not built for the QMX: its speaker volume and break-in are not changed by CWLD.
            RadioCommand::SetAfLevel(_) | RadioCommand::SetBreakIn(_) | RadioCommand::SetSquelch(_) => Err(RigError::InvalidArgument("not_supported".into())),
        }
    }

    fn fast_poll(&mut self) -> Vec<RigEvent> {
        if self.read_incoming().is_err() {
            return std::mem::take(&mut self.events);
        }
        // Ask the transmit state every time, the SWR only while transmitting.
        let _ = self.send("TQ;");
        if self.status.transmitting {
            let _ = self.send("SW;");
        }
        std::mem::take(&mut self.events)
    }

    fn poll(&mut self) -> Vec<RigEvent> {
        if self.read_incoming().is_err() {
            return std::mem::take(&mut self.events);
        }
        let now = self.clock.now_ms();
        let silent_since = self.last_rx.unwrap_or(self.started);
        if self.status.connected && now.saturating_sub(silent_since) > 3_000 {
            self.status.connected = false;
            self.events.push(RigEvent::Connected(false));
            self.events.push(RigEvent::Warning("no_answer".into()));
        }
        if !self.asked_id {
            self.asked_id = true;
            // `AI2;` makes the radio report changes by itself. Then identify the radio.
            let _ = self.send("ID;AI2;");
        }
        let _ = self.execute(&RadioCommand::ReadStatus);
        let _ = self.send("TB;");
        let _ = self.pump();
        std::mem::take(&mut self.events)
    }

    fn status(&self) -> RadioStatus {
        self.status.clone()
    }
}
