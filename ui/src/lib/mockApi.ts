import type { Activation, ActivationStats, Api, AudioStatus, CheckIssue, ClusterProbe, ClusterSpot, ClusterStatus, DupeInfo, ExportResult, LogFilter, LogQso, CoreEvent, CoreFailure, InputDevice, LinkReport, LookupInfo, LookupStatus, PortInfo, SwrAlert, PracticeSignal, RadioSettings, SendOutcome, Settings, Snapshot, TentativeChar, Tuning, TxPolicy } from './types';
import { durationSeconds, playSidetone } from './morse';
import { adifText, bandOf } from './adif';
import { DEFAULT_CLUSTER_NODES } from './cluster';
import { checkExport, validGrid, validWwffRef } from './logcheck';

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
    audio: { input: '', tone_hz: 700, auto_start: true, record_dir: '', squelch: 3.5, auto_tone: true, gain_db: 0, wide_filter: false, fading: true, min_wpm: 5, max_wpm: 50, lock_wpm: 0, afc: true },
    radio: { kind: 'simulator', port: '', civ_address: 0xb6, baud: 115200, pipeline: false, auto_connect: true },
    last_policy: 'listen_only',
    first_run_done: false,
    signals_panel: false,
    new_tab_seconds: 5,
    band_scope: true,
    audio_waterfall: true,
    default_rst: '5NN',
    tabs_auto_close_minutes: 15,
    contest_mode: false,
    contest_nr: 1,
    user_words: [],
    profiles: [{ id: 'home', name: 'Home', suffix: '', grid: '', power: '', rig: '', antenna: '', comment: '' }, { id: 'portable', name: 'Portable', suffix: '/P', grid: '', power: '', rig: '', antenna: '', comment: '' }],
    profile: 'home',
    scope_size: 'medium',
    scope_size_contest: 'small',
    lookup: { qrz_enabled: false, qrz_user: '' },
    cluster: {
      enabled: false,
      nodes: DEFAULT_CLUSTER_NODES.map((n) => ({ ...n })),
      node: 'auto',
      skimmer: true,
      show_labels: true,
      max_age_minutes: 30,
    },
    link: { adif_enabled: false, adif_host: '127.0.0.1', adif_port: 2333, n1mm_enabled: false, n1mm_targets: '127.0.0.1:12060' },
    practice: { exercise: 'they_cq', wpm_min: 18, wpm_max: 26, strength: 'medium', noise: true, qsb: false, pileup: 3, minutes: 10, decoder_help: true, volume: 0.5, quiet_radio: true, paddle: { exercise: 'chars', stats: [], sessions: [] }, koch: { chars: 'KMRSUAPTLOWINJEF0YVG5/Q9ZH38B?427C1D6X', char_wpm: 20, eff_wpm: 12, tone_hz: 650, listen: 'groups', stats: [], sessions: [] } },
    contest: { active: '', sent_index: 0, session: 1, profiles: [], vars: [{ name: 'SEC', value: 'WLD' }, { name: 'ZONE', value: '14' }, { name: 'ITU', value: '27' }, { name: 'PWR', value: '20' }] },
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

  // ---- Logbook (in memory, the real one is SQLite in Rust) ----
  private qsos: LogQso[] = [];
  private nextQsoId = 1;

  private act: Activation | null = null;
  private acts: Activation[] = [];
  private nextActId = 1;
  private nowSec = () => Math.floor(Date.now() / 1000);

  async logAdd(q: LogQso): Promise<LogQso> {
    const freq = q.freq_hz || this.status.frequency_hz;
    // The station the QSO is made from, as the Rust core fills it in.
    const st = this.settings;
    const base = st.station.calls[0] ?? q.my_call;
    const prof = st.profiles.find((p) => p.id === st.profile);
    const a = q.test ? null : this.act;
    const fill = (cur: string, v: string) => (cur.trim() ? cur : v);
    q = {
      ...q,
      station_call: fill(q.station_call, `${base}${prof?.suffix ?? ''}`.toUpperCase()),
      operator: fill(q.operator, base.toUpperCase()),
      activation_id: a ? a.id : q.activation_id,
      my_wwff: fill(q.my_wwff, a?.wwff ?? ''),
      my_pota: fill(q.my_pota, a?.pota ?? ''),
      my_sota: fill(q.my_sota, a?.sota ?? ''),
      my_grid: fill(q.my_grid, (a?.grid || prof?.grid || st.station.locator).toUpperCase()),
      tx_pwr: fill(q.tx_pwr, a?.power || prof?.power || ''),
      my_rig: fill(q.my_rig, prof?.rig ?? ''),
      my_antenna: fill(q.my_antenna, prof?.antenna ?? ''),
      comment: fill(q.comment, prof?.comment ?? ''),
    };
    const stored: LogQso = { ...q, id: this.nextQsoId++, call: q.call.trim().toUpperCase(), ts: q.ts || Math.floor(Date.now() / 1000), freq_hz: freq, band: q.band || bandOf(freq), mode: q.mode || 'CW', deleted: false };
    this.qsos.push(stored);
    return { ...stored };
  }
  async logList(f: LogFilter): Promise<LogQso[]> {
    let r = this.qsos.filter((q) => (f.include_deleted || !q.deleted) && (f.from_ts == null || q.ts >= f.from_ts) && (f.to_ts == null || q.ts <= f.to_ts) && (!f.band || q.band === f.band) && (!f.contest || q.contest === f.contest) && (f.session == null || q.session === f.session) && (f.test == null || q.test === f.test) && (f.activation == null || q.activation_id === f.activation) && (!f.not_exported || !q.exported_to.includes('adif')));
    r = r.sort((a, b) => b.ts - a.ts || b.id - a.id);
    return (f.limit ? r.slice(0, f.limit) : r).map((q) => ({ ...q }));
  }
  async logUpdate(q: LogQso): Promise<void> {
    const i = this.qsos.findIndex((x) => x.id === q.id);
    if (i >= 0) this.qsos[i] = { ...this.qsos[i], ...q, call: q.call.trim().toUpperCase() };
  }
  async logDelete(id: number): Promise<void> {
    const q = this.qsos.find((x) => x.id === id);
    if (q) q.deleted = true;
  }
  async logClearTest(): Promise<number> {
    const n = this.qsos.filter((q) => q.test).length;
    this.qsos = this.qsos.filter((q) => !q.test);
    return n;
  }
  async logNextTestSerial(): Promise<number> {
    return Math.max(0, ...this.qsos.filter((q) => q.test && !q.deleted).map((q) => q.nr_sent)) + 1;
  }
  async logDupe(call: string, contest: string, session: number, test: boolean): Promise<DupeInfo> {
    const band = bandOf(this.status.frequency_hz);
    const same = this.qsos.filter((q) => !q.deleted && q.call === call.trim().toUpperCase() && q.contest === contest && q.session === session && q.test === test);
    const dupe = same.filter((q) => q.band === band && q.mode === 'CW').sort((a, b) => b.ts - a.ts)[0] ?? null;
    return { dupe: dupe ? { ...dupe } : null, other_bands: [...new Set(same.map((q) => q.band))].filter((b) => b !== band).sort() };
  }
  async logExport(f: LogFilter, contestId: string): Promise<ExportResult> {
    const list = await this.logList({ ...f, limit: 0 });
    if ((await this.logCheck(f)).some((i) => i.severity === 'error')) fail('export_check_failed');
    if (f.test !== true) for (const q of this.qsos) if (list.some((x) => x.id === q.id) && !q.test && !q.exported_to.includes('adif')) q.exported_to = q.exported_to ? `${q.exported_to} adif` : 'adif';
    return { path: '(browser)', count: list.length, text: adifText(list, contestId) };
  }
  /** Lookup in the browser demo and in tests: a fake QRZ with one known station. */
  lookupPassword = '';
  lookupCalls: string[] = [];
  lookupAnswers: Record<string, LookupInfo> = {
    ON4ZZ: { call: 'ON4ZZ', first_name: 'Jan', last_name: 'Peeters', qth: 'Gent', country: 'Belgium', grid: 'JO10', limited: false },
  };
  async lookupStatus(): Promise<LookupStatus> {
    const l = this.settings.lookup;
    return { enabled: l.qrz_enabled, user: l.qrz_user, has_password: this.lookupPassword !== '' };
  }
  async lookupSetPassword(password: string): Promise<void> {
    if (!this.settings.lookup.qrz_user.trim()) fail('lookup_no_user');
    if (!password) fail('lookup_no_password');
    this.lookupPassword = password;
  }
  async lookupClearPassword(): Promise<void> {
    this.lookupPassword = '';
  }
  async lookupTest(): Promise<{ subscriber: boolean }> {
    if (!this.lookupPassword) fail('lookup_no_password');
    return { subscriber: true };
  }
  async lookupCall(call: string): Promise<LookupInfo | null> {
    if (!this.settings.lookup.qrz_enabled) return null;
    this.lookupCalls.push(call);
    if (!this.lookupPassword) fail('lookup_no_password');
    return this.lookupAnswers[call.toUpperCase()] ?? null;
  }
  /** Calls of linkResendLast, for tests. */
  linkResends = 0;
  async linkResendLast(): Promise<LinkReport> {
    const last = this.qsos.filter((q) => !q.deleted && !q.test).sort((a, b) => b.ts - a.ts)[0];
    if (!last) fail('link_no_qso');
    this.linkResends++;
    const l = this.settings.link;
    return { sent: (l.adif_enabled ? 1 : 0) + (l.n1mm_enabled ? l.n1mm_targets.split(/\s+/).filter(Boolean).length : 0), errors: [] };
  }
  async logCheck(f: LogFilter): Promise<CheckIssue[]> {
    return checkExport(await this.logList({ ...f, limit: 0 }), this.acts, this.nowSec());
  }
  async activationStart(a: Activation): Promise<Activation> {
    const st = this.settings;
    const prof = st.profiles.find((p) => p.id === st.profile);
    const wwff = a.wwff.trim().toUpperCase();
    if (wwff && !validWwffRef(wwff)) fail('activation_wwff');
    let grid = (a.grid.trim() || prof?.grid || '').toUpperCase();
    if (!grid) {
      if (prof?.suffix || !st.station.locator) fail('activation_grid_missing');
    } else if (!validGrid(grid)) fail('activation_grid');
    if (this.act) this.act.ended_ts = this.nowSec();
    const n: Activation = { ...a, id: this.nextActId++, profile: st.profile, grid, wwff, pota: a.pota.trim().toUpperCase(), sota: a.sota.trim().toUpperCase(), power: a.power.trim() || prof?.power || '', started_ts: this.nowSec(), ended_ts: 0 };
    this.acts.push(n);
    this.act = n;
    return { ...n };
  }
  async activationEnd(): Promise<void> {
    if (this.act) this.act.ended_ts = this.nowSec();
    this.act = null;
  }
  async activationCurrent(): Promise<Activation | null> {
    return this.act ? { ...this.act } : null;
  }
  // ---- DX cluster (demo: a few invented spots around the VFO, nothing leaves the browser) ----
  private clusterState: ClusterStatus = { state: 'off', node: '', software: '', reason: '' };
  private clusterList: ClusterSpot[] = [];
  /** Commands the real core accepts: reading and filter commands only. */
  static readonly CLUSTER_OK = ['sh', 'show', 'set', 'unset', 'clear', 'accept', 'acc', 'reject', 'rej', 'help', 'bye', 'quit', 'q'];

  private spotStatus(call: string, band: string): ClusterSpot['status'] {
    const mine = this.qsos.filter((x) => !x.deleted && !x.test && x.call === call);
    if (mine.some((x) => x.band === band)) return 'worked';
    return mine.length ? 'other_band' : 'new';
  }
  private colorSpots(): ClusterSpot[] {
    this.clusterList = this.clusterList.map((s) => ({ ...s, status: this.spotStatus(s.call, s.band) }));
    return this.clusterList.map((s) => ({ ...s }));
  }
  async clusterStart(): Promise<ClusterStatus> {
    if (!this.settings.station.calls[0]?.trim()) fail('cluster_call_missing');
    const node = this.settings.cluster.nodes[0]?.name ?? 'demo';
    this.clusterState = { state: 'online', node, software: 'CC Cluster', reason: '' };
    const f = this.status.frequency_hz;
    const band = bandOf(f);
    const now = this.nowSec();
    const make = (call: string, off: number, snr: number, wpm: number): ClusterSpot => ({
      call, freq_hz: Math.round((f + off) / 100) * 100, band, spotter: 'DL1ABC-#', comment: `CW ${snr} dB ${wpm} WPM CQ`, snr, wpm, skimmer: true, ts: now, hits: 1, status: this.spotStatus(call, band),
    });
    this.clusterList = [make('PA3ABC', -2400, 24, 22), make('DL1XYZ', -900, 12, 26), make('OK1TEST', 600, 18, 20), make('SP9ZZ', 1700, 9, 24)];
    this.pending.push({ type: 'cluster_status', status: { ...this.clusterState } });
    this.pending.push({ type: 'cluster_spots', spots: this.colorSpots() });
    return { ...this.clusterState };
  }
  async clusterStop(): Promise<ClusterStatus> {
    this.clusterState = { state: 'off', node: '', software: '', reason: '' };
    this.clusterList = [];
    return { ...this.clusterState };
  }
  async clusterStatus(): Promise<ClusterStatus> {
    return { ...this.clusterState };
  }
  async clusterSend(line: string): Promise<void> {
    if (this.clusterState.state !== 'online') fail('cluster_not_connected');
    const first = line.trim().toLowerCase().split(/[\s/]/)[0];
    if (!MockApi.CLUSTER_OK.includes(first)) fail('cluster_command_not_allowed');
    this.pending.push({ type: 'cluster_lines', lines: [`(demo) ${line.trim()}`] });
  }
  async clusterSpots(): Promise<ClusterSpot[]> {
    return this.colorSpots();
  }
  async clusterProbe(): Promise<ClusterProbe[]> {
    return this.settings.cluster.nodes.map((n, i) => ({ node: n.name, host: n.host, port: n.port, ok: i !== 2, millis: 40 + i * 25, software: i % 2 ? 'DX Spider' : 'CC Cluster', error: i === 2 ? 'connection refused' : '' }));
  }

  async activationStats(id: number): Promise<ActivationStats> {
    const q = this.qsos.filter((x) => !x.deleted && !x.test && x.activation_id === id);
    const unique = new Set(q.map((x) => `${x.call}|${x.band}|${x.mode}`)).size;
    const a = this.acts.find((x) => x.id === id);
    const end = a ? (a.ended_ts || this.nowSec()) : 0;
    return { total: q.length, unique, crosses_midnight: !!a && Math.floor(a.started_ts / 86400) !== Math.floor(end / 86400) };
  }
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

  async setTuning(t: Tuning) {
    this.settings.audio = {
      ...this.settings.audio,
      gain_db: Math.min(20, Math.max(-20, t.gain_db)),
      wide_filter: t.wide_filter,
      min_wpm: t.min_wpm,
      max_wpm: t.max_wpm,
      lock_wpm: t.lock_wpm,
    };
  }

  async setAfc(on: boolean) {
    this.settings.audio.afc = on;
  }

  async setFading(on: boolean) {
    this.settings.audio.fading = on;
  }

  strongestCalls = 0;
  async goStrongest() {
    this.strongestCalls++;
  }

  async setSquelch(value: number) {
    this.settings.audio.squelch = Math.min(8, Math.max(1, value));
  }

  /** Test helper: the version the pretend GitHub offers (null: none). */
  offered: { version: string; notes: string } | null = null;
  installed = false;
  async appVersion() {
    return '0.0.0-demo';
  }
  async checkUpdate() {
    return this.offered;
  }
  async installUpdate(onProgress: (f: number) => void) {
    onProgress(0.5);
    onProgress(1);
    this.installed = true;
  }

  async setAutoTone(on: boolean) {
    this.settings.audio.auto_tone = on;
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
    if (this.policy === 'practice' && policy !== 'practice') void this.practiceStop();
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

  // ---- Practice band: the stations are "decoded" perfectly after their time on air. ----
  practiceRunning = false;
  private practiceId = 1;
  private practiceTimers = new Map<number, ReturnType<typeof setTimeout>>();
  /** Every transmission keyed on the practice band, for tests. */
  practiceLog: PracticeSignal[] = [];

  async practiceStart(_noise: boolean, _volume: number, _seed: number): Promise<AudioStatus> {
    if (this.policy !== 'practice') fail('practice_mode_needed');
    this.practiceRunning = true;
    this.audio = { ...this.audio, running: true, error: null, source: 'practice band' };
    this.pending.push({ type: 'audio', status: { ...this.audio } });
    return { ...this.audio };
  }

  async practiceStop() {
    if (!this.practiceRunning) return;
    await this.practiceClear();
    this.practiceRunning = false;
    this.audio = { ...this.audio, running: false };
    this.pending.push({ type: 'audio', status: { ...this.audio } });
  }

  async practiceKey(sig: PracticeSignal): Promise<number> {
    if (this.policy !== 'practice') fail('practice_mode_needed');
    if (!this.practiceRunning) fail('practice_not_running');
    if (sig.text.length > 200) fail('message_too_long');
    const id = this.practiceId++;
    this.practiceLog.push({ ...sig });
    const secs = Math.max(0, sig.delay_s) + durationSeconds(sig.text, Math.max(5, Math.round(sig.wpm)));
    this.practiceTimers.set(id, setTimeout(() => this.practiceEnd(id, sig.text), secs * 1000));
    // In the browser the stations are heard through the page (the app plays them in the core).
    if (typeof window !== 'undefined' && !(globalThis as { __vitest_worker__?: unknown }).__vitest_worker__) {
      setTimeout(() => playSidetone(sig.text, Math.round(sig.wpm), sig.tone_hz), Math.max(0, sig.delay_s) * 1000);
    }
    return id;
  }

  /** Test helper: ends a transmission now, as if its time on air was over. */
  practiceEnd(id: number, text?: string) {
    const tm = this.practiceTimers.get(id);
    if (tm === undefined) return;
    clearTimeout(tm);
    this.practiceTimers.delete(id);
    if (text) this.pending.push({ type: 'decode', source: 'rx', text: text.toUpperCase() + ' ', conf: [...text, ' '].map(() => 1), tentative: [], wpm: 20 });
    this.pending.push({ type: 'practice_done', ids: [id] });
  }

  /** Test helper: ends every transmission on the band now. */
  practiceEndAll() {
    const texts = new Map(this.practiceLog.map((s, i) => [i + 1, s.text]));
    for (const id of [...this.practiceTimers.keys()]) this.practiceEnd(id, texts.get(id));
  }

  async practiceClear() {
    const ids = [...this.practiceTimers.keys()];
    for (const tm of this.practiceTimers.values()) clearTimeout(tm);
    this.practiceTimers.clear();
    if (ids.length) this.pending.push({ type: 'practice_done', ids });
  }

  async practiceVolume(_v: number) {}
  /** The paddle trainer asked for key times. */
  keying = false;
  async keyingCapture(on: boolean) {
    this.keying = on;
  }

  /** Delivers these events to the listeners at once (tests). */
  emitNow(events: CoreEvent[]) {
    this.handlers.forEach((h) => h(events));
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
    if (this.settings.band_scope && this.settings.radio.kind === 'simulator') ev.push(this.scopeSweep());
    if (ev.length) this.handlers.forEach((h) => h(ev));
  }

  private scopeTick = 0;

  /** A made-up band scope sweep, ±25 kHz around the VFO, like the simulator in the core. */
  scopeSweep(): CoreEvent {
    const c = this.status.frequency_hz;
    const lo = c - 25_000;
    const hi = c + 25_000;
    const stations: [number, number][] = [[7_012_300, 120], [7_018_000, 70], [7_023_600, 95], [7_025_400, 140], [7_041_200, 85]];
    this.scopeTick++;
    const data: number[] = [];
    for (let i = 0; i < 475; i++) {
      const f = lo + (i * (hi - lo)) / 475;
      let v = 18 + Math.floor(Math.random() * 14);
      stations.forEach(([sf, level], k) => {
        const keyed = Math.floor(this.scopeTick / (2 + k)) % 3 !== 0;
        const d = (f - sf) / 120;
        if (keyed) v = Math.max(v, level * Math.exp(-d * d));
      });
      data.push(Math.min(160, Math.round(v)));
    }
    return { type: 'rig', event: { kind: 'scope', value: { lo_hz: lo, hi_hz: hi, centered: true, out_of_range: false, data } } };
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
