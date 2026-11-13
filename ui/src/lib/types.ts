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
  auto_connect: boolean;
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
  /** The decoder moves to the strongest signal by itself when its pitch is quiet. */
  auto_tone: boolean;
  /** Extra gain on the incoming audio, -20 to +20 dB (decoder and meter, not the recording). */
  gain_db: number;
  /** Double filter bandwidth, for drifting or chirpy signals. */
  wide_filter: boolean;
  /** The narrow width with steep edges, against a strong station close by. Not with wide. */
  steep_filter: boolean;
  /** Join letters a sender spelled out (D L 4 S A V becomes DL4SAV). */
  join_spelled: boolean;
  /** Read weak dots and dashes in fast fading (QSB). */
  fading: boolean;
  /** Slowest and fastest speed the decoder looks for (5 to 60 WPM). */
  min_wpm: number;
  max_wpm: number;
  /** Fixed speed in WPM. 0: follow the speed. */
  lock_wpm: number;
  /** Follow a drifting tone. */
  afc: boolean;
}

export type DefaultRst = '5NN' | '599';

export interface Tuning {
  gain_db: number;
  wide_filter: boolean;
  steep_filter: boolean;
  join_spelled: boolean;
  min_wpm: number;
  max_wpm: number;
  lock_wpm: number;
}

/** A keyed tone heard in the stretch that was decoded again. */
export interface HeardTone {
  hz: number;
  score_db: number;
}

/** The last seconds decoded again (see `Api.replay`). */
export interface ReplayResult {
  text: string;
  conf: number[];
  tone_hz: number;
  seconds: number;
  wpm: number;
  tones: HeardTone[];
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
  /** Seconds the frequency must stay put before a new tab starts (1 to 30). */
  new_tab_seconds: number;
  /** Show the radio's band scope (when the radio sends it). */
  band_scope: boolean;
  /** Show the narrow audio waterfall (what the decoder hears). */
  audio_waterfall: boolean;
  /** The signal report in chips: 5NN (default) or 599. */
  default_rst: DefaultRst;
  /** Close idle conversations after this many minutes. 0: never. */
  tabs_auto_close_minutes: number;
  /** Most conversation tabs kept open. 0: no limit. */
  tabs_max: number;
  /** Contest mode: {NR} is filled in. */
  contest_mode: boolean;
  /** The next serial number in contest mode. */
  contest_nr: number;
  /** Own abbreviations added to the code library. */
  user_words: UserWord[];
  /** Contest profiles of the operator, the active profile and own variables. */
  contest: ContestSettings;
  /** Stations (home, portable) and the one in use. */
  profiles: StationProfile[];
  profile: string;
  /** Height of the band scope outside and in contest mode. */
  scope_size: ScopeSize;
  scope_size_contest: ScopeSize;
  practice: PracticeSettings;
  link: LinkSettings;
  lookup: LookupSettings;
  cluster: ClusterSettings;
  /** Changed chip and key texts, own chips, CQ repeat and holding text while the paddle keys. */
  send_texts: SendTexts;
}

/** A chip the operator added. */
export interface OwnChip {
  id: string;
  label: string;
  template: string;
}

/** Texts behind the chips and function keys. Only changes are stored; a slot without an entry uses its built-in text. */
export interface SendTexts {
  /** Changed texts by slot: `chip.cq`, `pileup.F3`, `contest.cqww-cw.run.F2`, ... */
  texts: Record<string, string>;
  /** Changed button names by slot. */
  labels: Record<string, string>;
  own_chips: OwnChip[];
  /** Order of the chips by slot. Chips not in the list follow in their own order. */
  chip_order: string[];
  /** Seconds of listening between two calls when CQ repeats (2 to 60). */
  cq_pause_s: number;
  /** Most calls in one go (1 to 30). */
  cq_max: number;
  /** Hold typed text while the paddle keys. */
  paddle_wait: boolean;
}

/** One DX cluster node. */
export interface ClusterNode {
  name: string;
  host: string;
  port: number;
}

/** DX cluster spots on the band scope (read only). Off by default. */
export interface ClusterSettings {
  /** Connect when the app starts. */
  enabled: boolean;
  nodes: ClusterNode[];
  /** `auto` or `host:port` of one node. */
  node: string;
  /** Also ask for skimmer (RBN) spots. */
  skimmer: boolean;
  /** Show the spot labels on the band scope. */
  show_labels: boolean;
  /** A spot disappears after this many minutes (1 to 240). */
  max_age_minutes: number;
}

export type ClusterState = 'off' | 'connecting' | 'online' | 'offline';

export interface ClusterStatus {
  state: ClusterState;
  node: string;
  software: string;
  /** Why the last connection ended. */
  reason: string;
}

/** `new`: not in the log. `other_band`: worked on another band. `worked`: worked on this band (a dupe in contest mode). */
export type SpotStatus = 'new' | 'other_band' | 'worked';

export interface ClusterSpot {
  call: string;
  freq_hz: number;
  band: string;
  spotter: string;
  comment: string;
  /** The last comment a person (not a skimmer) gave, kept for a while: "UP 1" lives here. */
  note?: string;
  snr: number | null;
  wpm: number | null;
  skimmer: boolean;
  /** Seconds since 1970 when the newest report arrived. */
  ts: number;
  hits: number;
  status: SpotStatus;
}

export interface ClusterProbe {
  node: string;
  host: string;
  port: number;
  ok: boolean;
  millis: number;
  software: string;
  error: string;
}

/** Sending logged QSOs over UDP to other programs. All off by default. */
export interface LinkSettings {
  adif_enabled: boolean;
  adif_host: string;
  adif_port: number;
  n1mm_enabled: boolean;
  /** `host:port` entries separated by spaces. */
  n1mm_targets: string;
}

export interface LookupSettings {
  qrz_enabled: boolean;
  /** QRZ.com login name. The password is never part of the settings: the operating system keeps it. */
  qrz_user: string;
}

export interface LookupStatus {
  enabled: boolean;
  user: string;
  has_password: boolean;
}

export interface LookupInfo {
  call: string;
  first_name: string;
  last_name: string;
  qth: string;
  country: string;
  grid: string;
  limited: boolean;
}

export interface LinkReport {
  sent: number;
  errors: string[];
}

export type ScopeSize = 'small' | 'medium' | 'large';

/** Choices of the practice simulator (see the Rust struct PracticeSettings). */
export interface PracticeSettings {
  exercise: 'they_cq' | 'you_cq' | 'contest';
  wpm_min: number;
  wpm_max: number;
  strength: 'weak' | 'medium' | 'strong';
  noise: boolean;
  qsb: boolean;
  pileup: number;
  minutes: number;
  decoder_help: boolean;
  volume: number;
  /** Practice mode turns the radio's speaker down and break-in off, and puts both back after. */
  quiet_radio: boolean;
  paddle: PaddleProgress;
  koch: KochSettings;
}

/** Koch method choices (see the Rust struct KochSettings). */
export interface KochSettings {
  /** Characters in use, in Koch order. */
  chars: string;
  char_wpm: number;
  eff_wpm: number;
  tone_hz: number;
  listen: 'groups' | 'words';
  stats: { ch: string; tries: number; errors: number }[];
  sessions: ListenSession[];
}

export interface ListenSession {
  ts: number;
  accuracy: number;
  n: number;
  chars: number;
}

export interface PaddleProgress {
  exercise: string;
  stats: { ch: string; tries: number; errors: number }[];
  sessions: PaddleSession[];
}

export interface PaddleSession {
  ts: number;
  exercise: string;
  score: number;
  wpm: number;
  accuracy: number;
}

/** One transmission of a simulated station on the practice band. */
export interface PracticeSignal {
  text: string;
  wpm: number;
  tone_hz: number;
  snr_db: number;
  qsb_depth: number;
  jitter: number;
  delay_s: number;
  seed: number;
}

export interface StationProfile {
  id: string;
  name: string;
  /** Added to the base call: empty at home, /P portable. */
  suffix: string;
  grid: string;
  power: string;
  rig: string;
  antenna: string;
  comment: string;
}

export interface ContestVar {
  name: string;
  value: string;
}

export interface SentTemplate {
  /** Plain text, or `i18n:key` for a built-in label. */
  label: string;
  template: string;
}

export interface RecvField {
  name: string;
  label: string;
  kind: 'text' | 'number' | 'choice';
  choices: string[];
  /** The field applies only to calls starting with one of these. Empty: always. */
  only_prefixes: string[];
}

export interface ContestProfile {
  id: string;
  name: string;
  note: string;
  calls: string[];
  sent: SentTemplate[];
  recv: RecvField | null;
  /** Send the serial number as cut numbers (0 as T, 9 as N). */
  cut_numbers: boolean;
}

export interface ContestSettings {
  /** Id of the active profile (built-in or own). Empty: none. */
  active: string;
  /** Which sent template of the active profile is used. */
  sent_index: number;
  profiles: ContestProfile[];
  vars: ContestVar[];
  /** Which run of the contest this is; a new contest raises it. */
  session: number;
}

export interface UserWord {
  code: string;
  meaning: string;
}

export interface RadioStatus {
  connected: boolean;
  frequency_hz: number;
  mode: string;
  filter: number | null;
  cw_wpm: number;
  transmitting: boolean;
  s_meter: number;
  /** Split: the radio transmits on `tx_hz` instead of where it receives. */
  split?: SplitState;
}

export interface SplitState {
  on: boolean;
  /** The transmit frequency in Hz, when the radio (or CWLD) knows it. */
  tx_hz: number | null;
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
  | { kind: 'swr'; value: number }
  | { kind: 'scope'; value: ScopeSweep }
  | { kind: 'split'; value: SplitState };

/** One sweep of the radio's own band scope: levels (0 to 160) evenly spread from lo_hz to hi_hz. */
export interface ScopeSweep {
  lo_hz: number;
  hi_hz: number;
  centered: boolean;
  out_of_range: boolean;
  data: number[];
}

export interface DecoderStats {
  /** 0..1, estimate from the decoder's own character probabilities. */
  confidence: number;
  snr_db: number | null;
  /** Real share of correct characters, only in practice mode where the keyed text is known. */
  accuracy: number | null;
  /** The decoder filter bandwidth in Hz (the filter setting, not a measurement). */
  bandwidth_hz?: number | null;
}

export type CoreEvent =
  | { type: 'rig'; event: RigEvent }
  | { type: 'policy'; policy: TxPolicy }
  | { type: 'decoder'; stats: DecoderStats }
  | { type: 'decode'; source: 'rx' | 'paddle'; text: string; conf: number[]; tentative: TentativeChar[]; wpm: number }
  | { type: 'waterfall'; bins: number[]; lo_hz: number; hz_per_bin: number }
  | { type: 'audio_level'; dbfs: number; peak: number }
  | { type: 'bandwidth'; hz: number }
  | { type: 'audio'; status: AudioStatus }
  | { type: 'swr_alert'; alert: SwrAlert }
  | { type: 'swr_cleared' }
  | { type: 'practice_done'; ids: number[] }
  /** Why the decoder shows nothing (`null`: it decodes again). `mode` is the radio mode, for example USB. */
  | { type: 'decode_hold'; hold: 'mode' | 'voice' | null; mode: string }
  /** Key down and up times in ms for the paddle trainer. */
  | { type: 'keying'; runs: [boolean, number][] }
  | { type: 'radio_quiet'; quiet: boolean }
  /** What the radio reports about its volume (0 to 255), squelch (0 open to 255 closed) and break-in. `null`: not reported. */
  | { type: 'radio_controls'; controls: RadioControls }
  /** A hole in the audio around this pitch (a notch on the radio?). `null`: gone. */
  | { type: 'audio_hole'; hz: number | null }
  | { type: 'cluster_status'; status: ClusterStatus }
  | { type: 'cluster_spots'; spots: ClusterSpot[] }
  | { type: 'cluster_lines'; lines: string[] };

export interface RadioControls {
  af_level: number | null;
  break_in: number | null;
  squelch: number | null;
  /** Manual notch (MN) and automatic notch (AN) on the radio. `null`: not reported. */
  notch?: boolean | null;
  auto_notch?: boolean | null;
  /** Twin PBT knobs, 0 to 255; 128 is the middle. */
  pbt_inner?: number | null;
  pbt_outer?: number | null;
}

export type SendOutcome = { kind: 'sent' } | { kind: 'simulated'; seconds: number };

/** Error as delivered by the core: a stable code, the interface translates it. */
export interface CoreFailure {
  code: string;
  message?: string;
}

/** A newer version found on GitHub. */
export interface UpdateInfo {
  version: string;
  notes: string;
}

/** One logged contact (see the Rust struct Qso). */
export interface LogQso {
  id: number;
  /** Seconds since 1970, UTC. 0: the core fills in the time. */
  ts: number;
  call: string;
  band: string;
  freq_hz: number;
  mode: string;
  rst_sent: string;
  rst_rcvd: string;
  nr_sent: number;
  nr_rcvd: string;
  extra_sent: string;
  extra_rcvd: string;
  name: string;
  qth: string;
  comment: string;
  /** Id of the contest profile, empty for a normal QSO. */
  contest: string;
  session: number;
  my_call: string;
  test: boolean;
  deleted: boolean;
  station_call: string;
  operator: string;
  my_grid: string;
  tx_pwr: string;
  my_rig: string;
  my_antenna: string;
  activation_id: number;
  my_wwff: string;
  my_pota: string;
  my_sota: string;
  their_prog: string;
  their_ref: string;
  exported_to: string;
}

export interface Activation {
  id: number;
  profile: string;
  grid: string;
  power: string;
  wwff: string;
  pota: string;
  sota: string;
  note: string;
  started_ts: number;
  ended_ts: number;
}

export interface ActivationStats {
  total: number;
  unique: number;
  crosses_midnight: boolean;
}

export interface CheckIssue {
  severity: 'error' | 'warning';
  code: string;
  id: number;
  detail: string;
}

export interface LogFilter {
  from_ts?: number | null;
  to_ts?: number | null;
  band?: string;
  contest?: string;
  session?: number | null;
  /** Undefined or null: both logs. False: the real log. True: the test log. */
  test?: boolean | null;
  include_deleted?: boolean;
  activation?: number | null;
  not_exported?: boolean;
  /** Part of the callsign, name, QTH, comment or reference (any case). */
  search?: string;
  limit?: number;
}

export interface DupeInfo {
  dupe: LogQso | null;
  other_bands: string[];
}

export interface ExportResult {
  path: string;
  count: number;
  text: string;
}

export function emptyLogQso(): LogQso {
  return { id: 0, ts: 0, call: '', band: '', freq_hz: 0, mode: 'CW', rst_sent: '', rst_rcvd: '', nr_sent: 0, nr_rcvd: '', extra_sent: '', extra_rcvd: '', name: '', qth: '', comment: '', contest: '', session: 0, my_call: '', test: false, deleted: false, station_call: '', operator: '', my_grid: '', tx_pwr: '', my_rig: '', my_antenna: '', activation_id: 0, my_wwff: '', my_pota: '', my_sota: '', their_prog: '', their_ref: '', exported_to: '' };
}

export interface Api {
  /** The version of the installed app. */
  appVersion(): Promise<string>;
  /** Asks GitHub for a newer version. Null when this is the newest. Sends no personal data. */
  checkUpdate(): Promise<UpdateInfo | null>;
  /** Downloads, installs and restarts. Progress is 0 to 1. */
  installUpdate(onProgress: (fraction: number) => void): Promise<void>;
  getSnapshot(): Promise<Snapshot>;
  setPolicy(policy: TxPolicy, confirmed: boolean): Promise<void>;
  sendText(text: string): Promise<SendOutcome>;
  stopTx(): Promise<void>;
  setSpeed(wpm: number): Promise<void>;
  setFrequency(hz: number): Promise<void>;
  /** Split on: transmit on `txHz`. With `rxHz` the radio first tunes there. Does not key the radio. */
  splitSet(txHz: number, rxHz?: number): Promise<void>;
  /** Split off: transmit where the radio receives. */
  splitOff(): Promise<void>;
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
  setAutoTone(on: boolean): Promise<void>;
  /** Gain, filter width and speed range of the running decoder. */
  setTuning(t: Tuning): Promise<void>;
  setAfc(on: boolean): Promise<void>;
  setFading(on: boolean): Promise<void>;
  /** Once: the decoder moves to the strongest keyed tone heard lately. */
  goStrongest(): Promise<void>;
  /** Decodes the last `seconds` (5 to 60) again with a separate decoder: at the live pitch
   * (`toneHz` null), at `toneHz`, or at the strongest tone of that stretch. */
  replay(seconds: number, toneHz: number | null, strongest: boolean): Promise<ReplayResult>;
  /** Saves the last minute of audio as a WAV file. Returns its path. */
  saveRewind(): Promise<string>;
  /** Returns the path of the new recording. */
  startRecording(): Promise<string>;
  stopRecording(): Promise<void>;
  logAdd(qso: LogQso): Promise<LogQso>;
  logList(filter: LogFilter): Promise<LogQso[]>;
  logUpdate(qso: LogQso): Promise<void>;
  logDelete(id: number): Promise<void>;
  /** Wipes the test log. Returns how many QSOs were removed. */
  logClearTest(): Promise<number>;
  logNextTestSerial(): Promise<number>;
  logDupe(call: string, contest: string, session: number, test: boolean): Promise<DupeInfo>;
  /** Writes an ADIF file in the CWLD folder. `contestId` is the ADIF CONTEST_ID. */
  logExport(filter: LogFilter, contestId: string): Promise<ExportResult>;
  /** Switches the manual and automatic notch of the radio off. */
  notchOff(): Promise<void>;
  /** Puts both Twin PBT knobs of the radio in the middle. */
  pbtCenter(): Promise<void>;
  /** Opens the folder with the exported files. Returns its path. */
  /** Writes a CSV in the export folder. Returns the path, or '(browser)' in the demo. */
  exportTextFile(name: string, text: string): Promise<string>;
  openExportFolder(): Promise<string>;
  /** Sends the newest real QSO again to the enabled UDP links (a connection test). */
  linkResendLast(): Promise<LinkReport>;
  lookupStatus(): Promise<LookupStatus>;
  lookupSetPassword(password: string): Promise<void>;
  lookupClearPassword(): Promise<void>;
  lookupTest(): Promise<{ subscriber: boolean }>;
  lookupCall(call: string): Promise<LookupInfo | null>;
  /** What an export of these QSOs would contain and what is wrong with it. */
  logCheck(filter: LogFilter): Promise<CheckIssue[]>;
  /** Starts the practice band in place of the radio audio (practice mode only). */
  practiceStart(noise: boolean, volume: number, seed: number): Promise<AudioStatus>;
  practiceStop(): Promise<void>;
  /** One transmission of a simulated station. Returns its id for the practice_done event. */
  practiceKey(signal: PracticeSignal): Promise<number>;
  practiceClear(): Promise<void>;
  practiceVolume(volume: number): Promise<void>;
  /** The paddle trainer wants the key times of the incoming audio (`keying` events), or not. */
  keyingCapture(on: boolean): Promise<void>;
  activationStart(a: Activation): Promise<Activation>;
  activationEnd(): Promise<void>;
  activationCurrent(): Promise<Activation | null>;
  activationStats(id: number): Promise<ActivationStats>;
  /** Connects to the DX cluster of the settings (needs an own callsign). */
  clusterStart(): Promise<ClusterStatus>;
  clusterStop(): Promise<ClusterStatus>;
  clusterStatus(): Promise<ClusterStatus>;
  /** A command typed in the cluster window. Only reading commands are accepted. */
  clusterSend(line: string): Promise<void>;
  /** All spots now, colored with the newest state of the logbook. */
  clusterSpots(): Promise<ClusterSpot[]>;
  /** Tries every node of the list. */
  clusterProbe(): Promise<ClusterProbe[]>;
  /** True for the browser stand-in: the conversation starts with example text. */
  readonly demo: boolean;
  /** Poll for events (the Tauri shell pushes them, the mock generates them). */
  onEvents(handler: (events: CoreEvent[]) => void): () => void;
}
