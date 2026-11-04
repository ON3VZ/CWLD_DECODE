import type { Api, AudioStatus, CoreEvent, CoreFailure, InputDevice, PortInfo, SwrAlert, RadioSettings, SendOutcome, Settings, Snapshot, TentativeChar, TxPolicy } from './types';
import { durationSeconds } from './morse';

/** Same rule as the Rust keyer: blocks of at most 30 characters, cut on word boundaries. */
export function chunkText(text: string, max = 30): string[] {
  const words = text.trim().toUpperCase().split(/\s+/).filter(Boolean);
  const out: string[] = [];
  let cur = '';
  for (const w of words) {
    if (w.length > max) throw { code: 'word_too_long' } satisfies CoreFailure;
    if (cur && cur.length + 1 + w.length + 1 > max) {
      out.push(cur + ' ');
      cur = w;
    } else {
      cur = cur ? `${cur} ${w}` : w;
    }
  }
  if (cur) out.push(cur);
  return out;
}

const fail = (code: string): never => {
  throw { code } satisfies CoreFailure;
};

export function defaultSettings(): Settings {
  return {
    schema_version: 1,
    language: 'en',
    theme: 'dark',
    station: { calls: [], name: '', qth: '', locator: '' },
    swr: { enabled: true, limit: 1.5, qrx_after_stop: true },
    audio: { input: '', tone_hz: 700, auto_start: true, record_dir: '' },
    radio: { kind: 'simulator', port: '', civ_address: 0xb6, baud: 115200, pipeline: false },
    last_policy: 'listen_only',
    first_run_done: false,
    signals_panel: false,
  };
}

/** Browser stand-in for the Rust core, with the same rules. Used for `vite dev`, tests and screenshots. */
export class MockApi implements Api {
  private policy: TxPolicy = 'listen_only';
  private settings: Settings = defaultSettings();
  private status = {
    connected: true,
    frequency_hz: 7_025_400,
    mode: 'cw',
    filter: 2 as number | null,
    cw_wpm: 22,
    transmitting: false,
    s_meter: 55,
  };
  private radioName = 'Simulated radio';
  private swr = 1.1;
  private swrOver = 0;
  private swrAlert: SwrAlert | null = null;
  private pending: CoreEvent[] = [];
  private txUntil = 0;
  private handlers = new Set<(e: CoreEvent[]) => void>();
  private timer: ReturnType<typeof setInterval> | null = null;
  private swrTimer: ReturnType<typeof setInterval> | null = null;
  /** Counts what a real radio would have received as transmit commands. */
  transmitCommands = 0;
  readonly demo = true;
  private audio: AudioStatus = {
    running: true, source: 'Demo', rate: 48000, tone_hz: 700, error: null,
    recording: false, record_seconds: 0, record_path: '', overruns: 0,
  };
  private recStarted = 0;

  constructor(initial?: Partial<Settings>) {
    this.settings = { ...this.settings, ...initial };
    this.policy = this.settings.last_policy === 'live' ? 'listen_only' : this.settings.last_policy;
  }

  async getSnapshot(): Promise<Snapshot> {
    return {
      policy: this.policy,
      swr_alert: this.swrAlert ? { ...this.swrAlert } : null,
      status: { ...this.status },
      radio_name: this.radioName,
      settings: structuredClone(this.settings),
      audio: { ...this.audio },
    };
  }

  async listAudioInputs(): Promise<InputDevice[]> {
    return [
      { name: 'Microphone (USB Audio CODEC )', default: false, suggested: true },
      { name: 'Microphone Array (Realtek Audio)', default: true, suggested: false },
    ];
  }

  async startAudio(): Promise<AudioStatus> {
    this.audio = { ...this.audio, running: true, error: null, source: this.settings.audio.input || 'Microphone (USB Audio CODEC )' };
    this.pending.push({ type: 'audio', status: { ...this.audio } });
    return { ...this.audio };
  }

  async stopAudio(): Promise<AudioStatus> {
    this.audio = { ...this.audio, running: false, recording: false };
    this.pending.push({ type: 'audio', status: { ...this.audio } });
    return { ...this.audio };
  }

  async decodeFile(path: string): Promise<AudioStatus> {
    if (!path.toLowerCase().endsWith('.wav')) fail('audio');
    this.audio = { ...this.audio, running: true, error: null, source: path };
    return { ...this.audio };
  }

  async setTone(hz: number) {
    const v = Math.min(1500, Math.max(200, hz));
    this.settings.audio.tone_hz = v;
    this.audio = { ...this.audio, tone_hz: v };
    this.pending.push({ type: 'audio', status: { ...this.audio } });
  }

  async startRecording(): Promise<string> {
    if (!this.audio.running) fail('audio');
    const khz = (this.status.frequency_hz / 1000).toFixed(1);
    this.recStarted = Date.now();
    this.audio = { ...this.audio, recording: true, record_seconds: 0, record_path: `Documents/CWLD/cwld-demo-${khz}kHz.wav` };
    this.pending.push({ type: 'audio', status: { ...this.audio } });
    return this.audio.record_path;
  }

  async stopRecording() {
    this.audio = { ...this.audio, recording: false, record_seconds: (Date.now() - this.recStarted) / 1000 };
    this.pending.push({ type: 'audio', status: { ...this.audio } });
  }

  /** Test helper: the decoder reports text. */
  decode(source: 'rx' | 'paddle', text: string, conf?: number[], tentative: TentativeChar[] = []) {
    this.pending.push({ type: 'decode', source, text, conf: conf ?? [...text].map(() => 1), tentative, wpm: 20 });
  }

  /** Test helper: one waterfall row. */
  waterfallRow(bins: number[], lo_hz = 200, hz_per_bin = 7.8125) {
    this.pending.push({ type: 'waterfall', bins, lo_hz, hz_per_bin });
  }

  async setPolicy(policy: TxPolicy, confirmed: boolean) {
    if (policy === 'live' && this.policy !== 'live' && !confirmed) fail('confirmation_required');
    if (this.policy === 'live' && policy !== 'live') this.endTx();
    this.policy = policy;
    this.settings.last_policy = policy;
  }

  async sendText(text: string): Promise<SendOutcome> {
    const clean = text.trim();
    if (!clean) fail('invalid_argument');
    if (this.policy === 'listen_only') fail('not_live');
    if (this.policy === 'live' && this.swrAlert) fail('swr_latched');
    const seconds = durationSeconds(clean, this.status.cw_wpm);
    if (seconds > 60) fail('message_too_long');
    const chunks = chunkText(clean);
    if (this.policy === 'practice') return { kind: 'simulated', seconds };
    this.transmitCommands += 1;
    this.status.transmitting = true;
    this.txUntil = Date.now() + seconds * 1000;
    this.pending.push({ type: 'rig', event: { kind: 'tx', value: true } });
    this.pending.push({ type: 'rig', event: { kind: 'queue', value: chunks.slice(1) } });
    return { kind: 'sent' };
  }

  async stopTx() {
    this.endTx();
  }

  private endTx() {
    if (this.status.transmitting) {
      this.status.transmitting = false;
      this.pending.push({ type: 'rig', event: { kind: 'tx', value: false } });
    }
    this.txUntil = 0;
    this.pending.push({ type: 'rig', event: { kind: 'queue', value: [] } });
  }

  async setSpeed(wpm: number) {
    if (wpm < 5 || wpm > 60) fail('invalid_argument');
    this.status.cw_wpm = wpm;
    this.pending.push({ type: 'rig', event: { kind: 'cw_speed', value: wpm } });
  }

  async setFrequency(hz: number) {
    if (hz < 30_000 || hz > 74_800_000) fail('frequency_out_of_range');
    this.status.frequency_hz = hz;
    this.pending.push({ type: 'rig', event: { kind: 'frequency', value: hz } });
  }

  async saveSettings(settings: Settings) {
    this.settings = structuredClone(settings);
  }

  async listPorts(): Promise<PortInfo[]> {
    return [
      { name: 'COM5', description: 'IC-7300 MK II (A)', vid: 0x0c26, pid: 0x0001, hint: 'icom' },
      { name: 'COM6', description: 'IC-7300 MK II (B)', vid: 0x0c26, pid: 0x0001, hint: 'icom' },
      { name: 'COM7', description: 'USB serial port', vid: 0x1234, pid: 0x5678, hint: null },
    ];
  }

  /** The browser cannot open a serial port: this pretends, so the screens can be tried. */
  async connect(radio: RadioSettings): Promise<Snapshot> {
    if (radio.kind !== 'simulator' && !radio.port.trim()) fail('port_missing');
    this.settings.radio = structuredClone(radio);
    this.radioName = radio.kind === 'icom' ? 'Icom IC-7300 MK II (demo)' : radio.kind === 'qmx' ? 'QRP Labs QMX (demo)' : 'Simulated radio';
    this.status.connected = true;
    return this.getSnapshot();
  }

  async disconnect(): Promise<Snapshot> {
    this.status.connected = false;
    this.radioName = '';
    return this.getSnapshot();
  }

  async ackSwr() {
    if (this.swrAlert) {
      this.swrAlert = null;
      this.swrOver = 0;
      this.pending.push({ type: 'swr_cleared' });
    }
  }

  /** Test and demo helper: the SWR the pretend radio reports while it transmits. */
  setSwr(swr: number) {
    this.swr = swr;
  }

  /** One SWR reading, as the real core gets them about every 100 ms. Same rule as the core. */
  swrReading(): CoreEvent[] {
    if (!this.status.transmitting) return [];
    const out: CoreEvent[] = [{ type: 'rig', event: { kind: 'swr', value: this.swr } }];
    const cfg = this.settings.swr;
    if (!cfg.enabled || this.swrAlert) return out;
    const limit = Math.min(3, Math.max(1.1, cfg.limit));
    this.swrOver = this.swr > limit ? this.swrOver + 1 : 0;
    if (this.swrOver >= 2) {
      this.endTx();
      this.swrAlert = { swr: this.swr, limit, qrx_sent: cfg.qrx_after_stop && this.policy === 'live' };
      this.swrOver = 0;
      out.push({ type: 'swr_alert', alert: { ...this.swrAlert } });
    }
    return out;
  }

  /** Delivers waiting events at once (tests do not want to wait for the 500 ms tick). */
  flush() {
    const events = this.pending;
    this.pending = [];
    if (events.length) this.handlers.forEach((h) => h(events));
  }

  /** Delivers one SWR reading (and a possible alert) to the listeners, like the fast tick of the core. */
  emitSwr() {
    const ev = this.swrReading();
    if (ev.length) this.handlers.forEach((h) => h(ev));
  }

  /** Test helper: text keyed with the paddle on the radio. */
  paddle(text: string) {
    this.pending.push({ type: 'rig', event: { kind: 'paddle_text', value: text } });
  }

  /** Simulates the operator turning the knob on the radio. */
  turnKnob(hz: number) {
    this.status.frequency_hz = hz;
    this.pending.push({ type: 'rig', event: { kind: 'frequency', value: hz } });
  }

  private conf = 0.93;
  private stats: import('./types').DecoderStats | null = null;

  /** Test and demo helper: fixes the decoder certainty the pretend decoder reports. */
  setDecoder(stats: import('./types').DecoderStats | null) {
    this.stats = stats;
    if (stats) this.pending.push({ type: 'decoder', stats });
  }

  private decoderTick() {
    if (this.stats) return;
    this.conf = Math.max(0.55, Math.min(0.99, this.conf + (Math.random() - 0.5) * 0.04));
    const stats = {
      confidence: this.conf,
      snr_db: 12 + (this.conf - 0.9) * 40,
      accuracy: this.policy === 'practice' ? Math.min(1, this.conf + 0.03) : null
    };
    this.pending.push({ type: 'decoder', stats });
  }

  private tick() {
    this.decoderTick();
    if (this.audio.recording) {
      this.audio = { ...this.audio, record_seconds: (Date.now() - this.recStarted) / 1000 };
      this.pending.push({ type: 'audio', status: { ...this.audio } });
    }
    if (this.status.transmitting && Date.now() >= this.txUntil) this.endTx();
    const wander = Math.round((Math.random() - 0.5) * 6);
    this.status.s_meter = Math.max(0, Math.min(100, this.status.s_meter + wander));
    this.pending.push({ type: 'rig', event: { kind: 's_meter', value: this.status.s_meter } });
    const events = this.pending;
    this.pending = [];
    this.handlers.forEach((h) => h(events));
  }

  onEvents(handler: (e: CoreEvent[]) => void) {
    this.handlers.add(handler);
    if (!this.timer) this.timer = setInterval(() => this.tick(), 500);
    if (!this.swrTimer) this.swrTimer = setInterval(() => this.emitSwr(), 100);
    return () => {
      this.handlers.delete(handler);
      if (this.handlers.size === 0 && this.timer) {
        clearInterval(this.timer);
        this.timer = null;
        if (this.swrTimer) clearInterval(this.swrTimer);
        this.swrTimer = null;
      }
    };
  }
}
