//! Driver for the Icom IC-7300 MK II over CI-V (USB virtual serial port).

use crate::civ::{self, cmd, Frame};
use crate::civ_scope::{self, ScopeAssembler};
use crate::clock::Clock;
use crate::error::RigError;
use crate::keyer::{self, KeyerError, ProsignStyle};
use crate::rig::{Capabilities, CwTextMode, Rig, RigInfo};
use crate::transport::Transport;
use cwld_types::{RadioCommand, RadioMode, RadioStatus, RigEvent, ScopeSweep};
use std::collections::VecDeque;

/// CI-V 0x17 takes at most 30 characters per command.
pub const CHUNK: usize = 30;
/// Extra time added to the estimated sending time of a block, in ms.
const MARGIN_MS: u64 = 200;
/// The radio must start transmitting within this time after a block was sent.
const TX_START_DEADLINE_MS: u64 = 2_000;
/// No frame from the radio for this long counts as "not connected".
const SILENCE_MS: u64 = 3_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ack {
    /// The acknowledgement of a CW block.
    Block,
    /// The acknowledgement of anything else we set.
    Other,
}

pub struct IcomRig<T: Transport> {
    t: T,
    clock: Box<dyn Clock>,
    parser: civ::Parser,
    radio: u8,
    pipeline: bool,
    status: RadioStatus,
    events: Vec<RigEvent>,
    queue: VecDeque<String>,
    last_block: Option<String>,
    acks: VecDeque<Ack>,
    busy_until: u64,
    ng_count: u8,
    tx_seen_since_send: bool,
    tx_deadline: Option<u64>,
    last_rx: Option<u64>,
    started: u64,
    scope: ScopeAssembler,
    /// The newest complete scope sweep. Only the newest is passed on, at most one per poll.
    scope_latest: Option<ScopeSweep>,
}

impl<T: Transport> IcomRig<T> {
    pub fn new(t: T, clock: Box<dyn Clock>, radio_address: u8, pipeline: bool) -> Self {
        let started = clock.now_ms();
        IcomRig {
            t,
            clock,
            parser: civ::Parser::default(),
            radio: radio_address,
            pipeline,
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
            queue: VecDeque::new(),
            last_block: None,
            acks: VecDeque::new(),
            busy_until: 0,
            ng_count: 0,
            tx_seen_since_send: false,
            tx_deadline: None,
            last_rx: None,
            started,
            scope: ScopeAssembler::default(),
            scope_latest: None,
        }
    }

    fn io_failed(&mut self, e: std::io::Error) -> RigError {
        if self.status.connected {
            self.status.connected = false;
            self.events.push(RigEvent::Connected(false));
        }
        RigError::Io(e.to_string())
    }

    fn send(&mut self, body: &[u8]) -> Result<(), RigError> {
        let frame = civ::encode(self.radio, civ::CONTROLLER, body);
        self.t.write_all(&frame).map_err(|e| self.io_failed(e))
    }

    fn send_set(&mut self, body: &[u8]) -> Result<(), RigError> {
        self.send(body)?;
        self.acks.push_back(Ack::Other);
        Ok(())
    }

    fn queue_event(&mut self) {
        self.events.push(RigEvent::Queue(self.queue.iter().cloned().collect()));
    }

    fn unit_ms(&self) -> u64 {
        1200 / u64::from(self.status.cw_wpm.max(1))
    }

    /// Sends the next block when the radio is free (or nearly free when pipelining).
    fn pump(&mut self) -> Result<(), RigError> {
        let Some(next) = self.queue.front() else { return Ok(()) };
        let now = self.clock.now_ms();
        let near_end = self.pipeline && now + 7 * self.unit_ms() >= self.busy_until;
        let busy = self.status.transmitting || now < self.busy_until;
        if busy && !near_end {
            return Ok(());
        }
        let block = next.clone();
        let mut body = vec![cmd::SEND_CW];
        body.extend_from_slice(block.as_bytes());
        self.send(&body)?;
        self.queue.pop_front();
        self.acks.push_back(Ack::Block);
        let est = keyer::estimate_ms(&block, self.status.cw_wpm, ProsignStyle::Caret) + MARGIN_MS;
        self.busy_until = self.busy_until.max(now) + est;
        self.last_block = Some(block);
        self.tx_seen_since_send = false;
        self.tx_deadline = Some(now + TX_START_DEADLINE_MS);
        self.queue_event();
        Ok(())
    }

    fn handle(&mut self, f: Frame) {
        if f.from != self.radio || f.body.is_empty() {
            return; // our own echo, or another device on the bus
        }
        let now = self.clock.now_ms();
        self.last_rx = Some(now);
        if !self.status.connected {
            self.status.connected = true;
            self.events.push(RigEvent::Connected(true));
        }
        let b = &f.body;
        match b[0] {
            cmd::FREQ_PUSH | cmd::READ_FREQ => {
                if let Some(hz) = civ::bcd_to_freq(&b[1..]) {
                    if hz != self.status.frequency_hz {
                        self.status.frequency_hz = hz;
                        self.events.push(RigEvent::Frequency(hz));
                    }
                }
            }
            cmd::MODE_PUSH | cmd::READ_MODE => {
                if b.len() >= 2 {
                    let m = civ::mode_from_code(b[1]);
                    if m != self.status.mode {
                        self.status.mode = m;
                        self.events.push(RigEvent::Mode(m));
                    }
                    if b.len() >= 3 {
                        self.status.filter = Some(b[2]);
                    }
                }
            }
            0x14 if b.len() >= 4 && b[1] == 0x0C => {
                if let Some(raw) = civ::from_bcd2(&b[2..]) {
                    let w = civ::raw_to_wpm(raw);
                    if w != self.status.cw_wpm {
                        self.status.cw_wpm = w;
                        self.events.push(RigEvent::CwSpeed(w));
                    }
                }
            }
            0x15 if b.len() >= 4 && b[1] == 0x02 => {
                if let Some(raw) = civ::from_bcd2(&b[2..]) {
                    // 0 to 255 to 0 to 100. The real scale is not the same everywhere **[TEST]**.
                    let v = ((u32::from(raw.min(255)) * 100) / 255) as u8;
                    if v != self.status.s_meter {
                        self.status.s_meter = v;
                        self.events.push(RigEvent::SMeter(v));
                    }
                }
            }
            0x15 if b.len() >= 4 && b[1] == 0x12 => {
                if let Some(raw) = civ::from_bcd2(&b[2..]) {
                    self.events.push(RigEvent::Swr(civ::swr_from_raw(raw)));
                }
            }
            0x1C if b.len() >= 3 && b[1] == 0x00 => {
                let tx = b[2] != 0;
                if tx != self.status.transmitting {
                    self.status.transmitting = tx;
                    self.events.push(RigEvent::Tx(tx));
                }
                if tx {
                    self.tx_seen_since_send = true;
                    self.tx_deadline = None;
                }
            }
            civ_scope::SCOPE if b.len() >= 2 && b[1] == civ_scope::SCOPE_DATA => {
                if let Some(s) = self.scope.push(b) {
                    self.scope_latest = Some(s);
                }
            }
            civ::OK => {
                self.acks.pop_front();
                self.ng_count = 0;
            }
            civ::NG => {
                if self.acks.pop_front() == Some(Ack::Block) {
                    self.on_block_refused();
                }
            }
            _ => {}
        }
    }

    fn take_events(&mut self) -> Vec<RigEvent> {
        if let Some(s) = self.scope_latest.take() {
            self.events.push(RigEvent::Scope(s));
        }
        std::mem::take(&mut self.events)
    }

    /// The radio said NG to a block. Put it back, and stop pipelining if that was the cause.
    fn on_block_refused(&mut self) {
        self.ng_count += 1;
        if let Some(b) = self.last_block.take() {
            self.queue.push_front(b);
        }
        self.busy_until = 0;
        if self.pipeline {
            self.pipeline = false;
            self.events.push(RigEvent::Warning("civ_pipeline_off".into()));
        } else if self.ng_count >= 3 {
            self.queue.clear();
            self.events.push(RigEvent::Warning("civ_rejected".into()));
        }
        self.queue_event();
    }
}

impl<T: Transport> Rig for IcomRig<T> {
    fn info(&self) -> RigInfo {
        RigInfo { name: "Icom IC-7300 MK II".to_string(), driver: "icom" }
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities { cw_text: CwTextMode::Chunked { max: CHUNK }, radio_decoder_text: false, dtr_keying: false, pushes_status: true, swr_meter: true, scope: true }
    }

    fn execute(&mut self, c: &RadioCommand) -> Result<(), RigError> {
        match c {
            RadioCommand::ReadStatus => {
                for body in [&[cmd::READ_FREQ][..], &[cmd::READ_MODE][..], &cmd::TX_STATUS[..], &cmd::KEYER_SPEED[..], &cmd::S_METER[..]] {
                    self.send(body)?;
                }
                Ok(())
            }
            RadioCommand::SetFrequency(hz) => {
                let mut body = vec![cmd::SET_FREQ];
                body.extend_from_slice(&civ::freq_to_bcd(*hz));
                self.send_set(&body)
            }
            RadioCommand::SetMode(m) => {
                let code = civ::mode_code(*m).ok_or_else(|| RigError::InvalidArgument("mode".into()))?;
                self.send_set(&[cmd::SET_MODE, code])
            }
            RadioCommand::SetCwSpeed(w) => {
                if !(civ::MIN_WPM..=civ::MAX_WPM).contains(w) {
                    return Err(RigError::InvalidArgument(format!("speed must be {} to {} WPM on this radio", civ::MIN_WPM, civ::MAX_WPM)));
                }
                let raw = civ::bcd2(civ::wpm_to_raw(*w));
                self.send_set(&[cmd::KEYER_SPEED[0], cmd::KEYER_SPEED[1], raw[0], raw[1]])
            }
            RadioCommand::SendCw(text) => {
                let prepared = keyer::prepare(text, ProsignStyle::Caret).map_err(|e| match e {
                    KeyerError::Empty => RigError::InvalidArgument("empty text".into()),
                    KeyerError::WordTooLong { .. } => RigError::InvalidArgument("word_too_long".into()),
                })?;
                let chunks = keyer::chunk_words(&prepared.words, CHUNK).map_err(|_| RigError::InvalidArgument("word_too_long".into()))?;
                let total: u64 = chunks.iter().map(|c| keyer::estimate_ms(c, self.status.cw_wpm, ProsignStyle::Caret)).sum();
                if total > keyer::MAX_MESSAGE_SECONDS * 1000 {
                    return Err(RigError::InvalidArgument("message_too_long".into()));
                }
                if !prepared.dropped.is_empty() {
                    self.events.push(RigEvent::Warning("dropped_chars".into()));
                }
                self.queue.extend(chunks);
                self.queue_event();
                self.pump()
            }
            RadioCommand::StopCw => {
                self.queue.clear();
                self.busy_until = 0;
                self.last_block = None;
                self.queue_event();
                self.send_set(&[cmd::SEND_CW, 0xFF])
            }
            RadioCommand::SetPtt(on) => self.send_set(&[cmd::TX_STATUS[0], cmd::TX_STATUS[1], u8::from(*on)]),
            RadioCommand::KeyDown(_) => Err(RigError::InvalidArgument("key_down is not supported by this driver".into())),
            RadioCommand::Raw(bytes) => self.t.write_all(bytes).map_err(|e| self.io_failed(e)),
            RadioCommand::SetScope(on) => {
                if *on {
                    self.send_set(&civ_scope::scope_on(true))?;
                }
                self.send_set(&civ_scope::scope_output(*on))
            }
        }
    }

    fn poll(&mut self) -> Vec<RigEvent> {
        match self.t.read_available() {
            Ok(bytes) => {
                for f in self.parser.push(&bytes) {
                    self.handle(f);
                }
            }
            Err(e) => {
                let _ = self.io_failed(e);
                return std::mem::take(&mut self.events);
            }
        }
        let now = self.clock.now_ms();
        let silent_since = self.last_rx.unwrap_or(self.started);
        if self.status.connected && now.saturating_sub(silent_since) > SILENCE_MS {
            self.status.connected = false;
            self.events.push(RigEvent::Connected(false));
            self.events.push(RigEvent::Warning("no_answer".into()));
        }
        if let Some(deadline) = self.tx_deadline {
            if now > deadline && !self.tx_seen_since_send {
                self.tx_deadline = None;
                self.events.push(RigEvent::Warning("radio_not_transmitting".into()));
            }
        }
        let _ = self.execute(&RadioCommand::ReadStatus);
        let _ = self.pump();
        self.take_events()
    }

    fn fast_poll(&mut self) -> Vec<RigEvent> {
        if let Ok(bytes) = self.t.read_available() {
            for f in self.parser.push(&bytes) {
                self.handle(f);
            }
        }
        // Ask the transmit state every time, and the SWR only while transmitting.
        let _ = self.send(&cmd::TX_STATUS);
        if self.status.transmitting {
            let _ = self.send(&cmd::SWR_METER);
        }
        self.take_events()
    }

    fn status(&self) -> RadioStatus {
        self.status.clone()
    }
}
