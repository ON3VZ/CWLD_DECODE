export type TxPolicy = 'live' | 'listen_only' | 'practice';
export type Language = 'en' | 'nl';
export type Theme = 'dark' | 'light' | 'system';

export interface SwrSettings {
  enabled: boolean;
  limit: number;
  qrx_after_stop: boolean;
}

export interface SwrAlert {
  swr: number;
  limit: number;
  qrx_sent: boolean;
}

export interface Station {
  calls: string[];
  name: string;
  qth: string;
  locator: string;
}

export type RadioKind = 'simulator' | 'icom' | 'qmx';

export interface RadioSettings {
  kind: RadioKind;
  port: string;
  civ_address: number;
  baud: number;
  pipeline: boolean;
}

export interface PortInfo {
  name: string;
  description: string;
  vid: number | null;
  pid: number | null;
  hint: RadioKind | null;
}

export interface AudioSettings {
  /** Sound card input name. Empty: a radio-looking input, else the system default. */
  input: string;
  tone_hz: number;
  auto_start: boolean;
  record_dir: string;
  /** 1 to 8. Higher: the decoder trusts only clearer signals. */
  squelch: number;
}

export interface AudioStatus {
  running: boolean;
  source: string;
  rate: number;
  tone_hz: number;
  error: string | null;
  recording: boolean;
  record_seconds: number;
  record_path: string;
  overruns: number;
}

export interface InputDevice {
  name: string;
  default: boolean;
  suggested: boolean;
}

export interface TentativeChar {
  best: string;
  confidence: number;
  alts: [string, number][];
}

export interface Settings {
  schema_version: number;
  language: Language;
  theme: Theme;
  station: Station;
  radio: RadioSettings;
  swr: SwrSettings;
  audio: AudioSettings;
  last_policy: TxPolicy;
  first_run_done: boolean;
  signals_panel: boolean;
}

export interface RadioStatus {
  connected: boolean;
  frequency_hz: number;
  mode: string;
  filter: number | null;
  cw_wpm: number;
  transmitting: boolean;
  s_meter: number;
}

export interface Snapshot {
  policy: TxPolicy;
  swr_alert: SwrAlert | null;
  status: RadioStatus;
  radio_name: string;
  settings: Settings;
  audio: AudioStatus;
}

export type RigEvent =
  | { kind: 'connected'; value: boolean }
  | { kind: 'frequency'; value: number }
  | { kind: 'mode'; value: string }
  | { kind: 'tx'; value: boolean }
  | { kind: 's_meter'; value: number }
  | { kind: 'cw_speed'; value: number }
  | { kind: 'paddle_text'; value: string }
  | { kind: 'queue'; value: string[] }
  | { kind: 'warning'; value: string }
  | { kind: 'swr'; value: number };

export interface DecoderStats {
  /** 0..1, estimate from the decoder's own character probabilities. */
  confidence: number;
  snr_db: number | null;
  /** Real share of correct characters, only in practice mode where the keyed text is known. */
  accuracy: number | null;
}

export type CoreEvent =
  | { type: 'rig'; event: RigEvent }
  | { type: 'policy'; policy: TxPolicy }
  | { type: 'decoder'; stats: DecoderStats }
  | { type: 'decode'; source: 'rx' | 'paddle'; text: string; conf: number[]; tentative: TentativeChar[]; wpm: number }
  | { type: 'waterfall'; bins: number[]; lo_hz: number; hz_per_bin: number }
  | { type: 'audio_level'; dbfs: number }
  | { type: 'audio'; status: AudioStatus }
  | { type: 'swr_alert'; alert: SwrAlert }
  | { type: 'swr_cleared' };

export type SendOutcome = { kind: 'sent' } | { kind: 'simulated'; seconds: number };

/** Error as delivered by the core: a stable code, the interface translates it. */
export interface CoreFailure {
  code: string;
  message?: string;
}

export interface Api {
  getSnapshot(): Promise<Snapshot>;
  setPolicy(policy: TxPolicy, confirmed: boolean): Promise<void>;
  sendText(text: string): Promise<SendOutcome>;
  stopTx(): Promise<void>;
  setSpeed(wpm: number): Promise<void>;
  setFrequency(hz: number): Promise<void>;
  saveSettings(settings: Settings): Promise<void>;
  listPorts(): Promise<PortInfo[]>;
  connect(radio: RadioSettings): Promise<Snapshot>;
  disconnect(): Promise<Snapshot>;
  /** The operator confirms an SWR alert. */
  ackSwr(): Promise<void>;
  listAudioInputs(): Promise<InputDevice[]>;
  startAudio(): Promise<AudioStatus>;
  stopAudio(): Promise<AudioStatus>;
  /** Decodes a WAV file at real speed, as if it came from the radio. */
  decodeFile(path: string): Promise<AudioStatus>;
  setTone(hz: number): Promise<void>;
  setSquelch(value: number): Promise<void>;
  /** Returns the path of the new recording. */
  startRecording(): Promise<string>;
  stopRecording(): Promise<void>;
  /** True for the browser stand-in: the conversation starts with example text. */
  readonly demo: boolean;
  /** Poll for events (the Tauri shell pushes them, the mock generates them). */
  onEvents(handler: (events: CoreEvent[]) => void): () => void;
}
