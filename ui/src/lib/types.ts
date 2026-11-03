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

export interface Settings {
  schema_version: number;
  language: Language;
  theme: Theme;
  station: Station;
  radio: RadioSettings;
  swr: SwrSettings;
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
  /** Poll for events (the Tauri shell pushes them, the mock generates them). */
  onEvents(handler: (events: CoreEvent[]) => void): () => void;
}
