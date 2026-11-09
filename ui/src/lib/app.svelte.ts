import type { Activation, ActivationStats, Api, DupeInfo, LogQso, AudioStatus, Tuning, CoreEvent, CoreFailure, DecoderStats, InputDevice, PortInfo, RadioSettings, RadioStatus, Settings, Snapshot, SwrAlert, TentativeChar, TxPolicy, UpdateInfo, ScopeSweep } from './types';
import { i18n, t } from './i18n.svelte';
import { activeProfile, activeSent, copyProfile as copyContestProfileData, cutNumber, isBuiltin, mergeImport, newProfileId, ownVars, parseImport, usesSerial, allProfiles, plain } from './contest';
import { demoMessages, demoSignals, pileupChips, contestChips, type Message, type Signal } from './demo';
import { expand } from './template';
import { bandOf } from './adif';
import { emptyLogQso } from './types';
import { contestKeys, enterAction, entryComplete, nextSerial, suggestFrom, type RunMode, type Suggestion } from './contestlog';
import { heardIn } from './qso';
import { PracticeSim, type Score, type SimTx } from './sim';
import type { PracticeSettings } from './types';
import { durationSeconds, playSidetone } from './morse';

export interface Qso {
  call: string;
  name: string;
  rst: string;
  qth: string;
}

const emptyQso = (rst = '5NN'): Qso => ({ call: '', name: '', rst, qth: '' });

/** Decoder strictness presets (values of the squelch slider). Chosen by feel, not yet measured. */
export const PRESETS = { weak: 2.5, medium: 3.5, strong: 5.5 } as const;
export type PresetName = keyof typeof PRESETS;

/** One conversation: what was heard and sent on one frequency, with its own QSO details. */
export interface Session {
  id: number;
  /** The frequency in Hz the conversation started on. 0: not known yet. */
  freq: number;
  started: string;
  messages: Message[];
  qso: Qso;
  /** True when the operator typed or clicked the callsign. Then the decoder stops overwriting it. */
  callLocked: boolean;
  /** Last activity (ms since 1970): text in or out, or the operator looked at it. */
  touched?: number;
  /** The QSO of this conversation is in the logbook. */
  logged?: boolean;
}

/** Closing idle conversations is checked this often (ms). */
export const SWEEP_MS = 30_000;

/** A frequency change of more than this many Hz starts a new conversation. */
export const NEW_CONVERSATION_HZ = 500;
/** Default wait before the frequency counts as a new conversation (ms), see settings.new_tab_seconds. */
export const SETTLE_MS = 5000;
/** Saved conversations kept on this computer. */
export const MAX_SESSIONS = 40;
const STORE_KEY = 'cwld.sessions.v1';

const clockNow = () => new Date().toTimeString().slice(0, 5);

export interface WaterfallRow {
  bins: number[];
  lo_hz: number;
  hz_per_bin: number;
}

/** A pause longer than this starts a new bubble for decoded text. */
export const BUBBLE_PAUSE_MS = 8000;

function codeOf(e: unknown): string {
  if (e && typeof e === 'object' && 'code' in e) return String((e as CoreFailure).code);
  return 'unknown';
}

class AppState {
  ready = $state(false);
  /** Running against the browser stand-in (example conversation and waterfall). */
  demo = $state(false);
  policy = $state<TxPolicy>('listen_only');
  status = $state<RadioStatus>({
    connected: false, frequency_hz: 0, mode: 'cw', filter: null, cw_wpm: 20, transmitting: false, s_meter: 0,
  });
  radioName = $state('');
  settings = $state<Settings | null>(null);
  /** All conversations, oldest first. The last one is the live one: new text always goes there. */
  sessions = $state<Session[]>([]);
  /** The conversation on screen. Equal to the live one unless the operator looks back. */
  viewId = $state(0);
  signals = $state<Signal[]>(demoSignals);
  draft = $state('');
  error = $state<string | null>(null);
  notice = $state<string | null>(null);
  confirmLive = $state(false);
  settingsOpen = $state(false);
  /** The guided first start. Opens by itself until it has been finished or skipped once. */
  wizardOpen = $state(false);
  /** Q-code library panel (slide-out). */
  qOpen = $state(false);
  /** Highest sample in the last tenth of a second, 0 to 1 and above (after the gain). */
  audioPeak = $state(0);
  /** The decoder filter bandwidth in Hz, for the lines in the audio waterfall. */
  bandwidthHz = $state<number | null>(null);
  /** More than 0: "close all" waits for a yes because this many conversations hold an unlogged QSO. */
  closeAllAsk = $state(0);
  /** The operator switches contest mode on while an earlier number is stored: continue or restart? */
  contestAsk = $state(false);
  /** Seconds left of a simulated transmission in practice mode. */
  practiceBusy = $state(false);
  /** Blocks waiting to be sent to the radio. */
  queue = $state<string[]>([]);
  /** The last SWR reading while transmitting, and the alert that blocks sending. */
  swr = $state<number | null>(null);
  swrAlert = $state<SwrAlert | null>(null);
  /** Decoder certainty for the meter next to the S-meter. Null: no decoded text yet. */
  decoder = $state<DecoderStats | null>(null);
  /** The latest warning from the radio layer, as a code. */
  warning = $state<string | null>(null);
  ports = $state<PortInfo[]>([]);
  connecting = $state(false);
  /** Audio input and decoder state. */
  audio = $state<AudioStatus | null>(null);
  audioLevel = $state<number | null>(null);
  inputs = $state<InputDevice[]>([]);
  /** Characters the decoder is not sure about yet, shown faded after the last bubble. */
  tail = $state<{ from: 'rx' | 'paddle'; chars: TentativeChar[] } | null>(null);
  /** Newest band scope sweeps, not yet drawn (the band scope component takes them). */
  scopeRows: ScopeSweep[] = [];
  /** True once the radio sends band scope data. */
  scopeLive = $state(false);
  /** Edges of the newest sweep, for the frequency scale and clicks. */
  scopeLo = $state(0);
  scopeHi = $state(0);
  scopeOutOfRange = $state(false);

  /** True once real waterfall rows arrive (otherwise the waterfall shows the demo). */
  waterfallLive = $state(false);
  /** Waterfall rows not drawn yet. Plain array on purpose: 20 rows a second need no reactivity. */
  waterfallRows: WaterfallRow[] = [];
  private lastDecodeAt: Record<'rx' | 'paddle', number> = { rx: 0, paddle: 0 };
  private lastPaddleAt = 0;

  private api!: Api;
  private off: (() => void) | null = null;
  private nextId = 100;
  private stopSidetone: (() => void) | null = null;
  private practiceTimer: ReturnType<typeof setTimeout> | null = null;

  /** The live conversation. */
  get cur(): Session {
    return this.sessions[this.sessions.length - 1];
  }

  /** The conversation on screen. */
  get shown(): Session {
    return this.sessions.find((x) => x.id === this.viewId) ?? this.cur;
  }

  get viewingLive(): boolean {
    return this.shown.id === this.cur.id;
  }

  get messages(): Message[] {
    return this.shown.messages;
  }
  set messages(v: Message[]) {
    this.shown.messages = v;
  }
  get qso(): Qso {
    return this.shown.qso;
  }
  set qso(v: Qso) {
    this.shown.qso = v;
  }
  get callLocked(): boolean {
    return this.shown.callLocked;
  }
  set callLocked(v: boolean) {
    this.shown.callLocked = v;
  }

  private blankQso(): Qso {
    return emptyQso(this.settings?.default_rst ?? '5NN');
  }

  private makeSession(freq: number): Session {
    return { id: this.nextId++, freq, started: clockNow(), messages: [], qso: this.blankQso(), callLocked: false, touched: Date.now() };
  }

  private touch(s: Session | undefined, now = Date.now()) {
    if (s) s.touched = now;
  }

  /**
   * Starts a new conversation and shows it. An empty live conversation is reused instead, so
   * turning the tuning knob does not leave a trail of empty tabs.
   */
  newConversation(freq = this.status.frequency_hz) {
    const live = this.cur;
    this.tail = null;
    if (live && live.messages.length === 0) {
      live.freq = freq;
      live.qso = this.blankQso();
      live.callLocked = false;
      this.viewId = live.id;
      return;
    }
    const next = this.makeSession(freq);
    this.sessions.push(next);
    while (this.sessions.length > MAX_SESSIONS) this.sessions.shift();
    this.viewId = next.id;
    this.persist();
  }

  /** The operator looks at another conversation. Nothing is lost in the live one. */
  show(id: number) {
    if (this.sessions.some((x) => x.id === id)) this.viewId = id;
  }

  /**
   * The operator clicks a tab. An earlier conversation becomes the live one again and the radio
   * goes to its frequency, so new text lands where it belongs. Without a radio that answers, or
   * while transmitting, the conversation is only shown.
   */
  async openTab(id: number) {
    const s = this.sessions.find((x) => x.id === id);
    if (!s) return;
    this.touch(s);
    if (id === this.cur.id) {
      this.viewId = id;
      return;
    }
    const canTune = this.status.connected && !this.status.transmitting && !this.practiceBusy && s.freq > 0;
    if (!canTune) {
      this.viewId = id;
      return;
    }
    const old = this.cur;
    this.sessions = this.sessions.filter((x) => x.id !== id && !(x.id === old.id && old.messages.length === 0));
    this.sessions.push(s);
    this.viewId = id;
    this.tail = null;
    if (this.settleTimer) {
      clearTimeout(this.settleTimer);
      this.settleTimer = null;
    }
    this.persist();
    if (Math.abs(this.status.frequency_hz - s.freq) >= 10) await this.setFrequency(s.freq);
  }

  /** A conversation with sent text and no logbook entry yet is not closed by itself. */
  private holdsUnloggedQso(s: Session): boolean {
    return !s.logged && s.messages.some((m) => m.from === 'tx');
  }

  /** Closes conversations that were idle longer than the setting. Never the live or the shown one. */
  sweepTabs(now = Date.now()) {
    const minutes = this.settings?.tabs_auto_close_minutes ?? 0;
    if (minutes <= 0 || this.sessions.length < 2) return;
    const limit = minutes * 60_000;
    const keep = this.sessions.filter((s) => {
      if (s.id === this.cur.id || s.id === this.viewId) return true;
      if (this.holdsUnloggedQso(s)) return true;
      if (s.messages.length === 0) return false;
      return now - (s.touched ?? now) < limit;
    });
    if (keep.length !== this.sessions.length) {
      this.sessions = keep;
      this.persist();
    }
  }

  /** "Close all": asks first when some conversations hold a QSO that is not in the logbook. */
  requestCloseAll() {
    const n = this.sessions.filter((s) => this.holdsUnloggedQso(s)).length;
    if (n > 0) this.closeAllAsk = n;
    else this.closeAllNow();
  }

  closeAllNow() {
    this.closeAllAsk = 0;
    this.tail = null;
    const fresh = this.makeSession(this.status.frequency_hz);
    this.sessions = [fresh];
    this.viewId = fresh.id;
    this.persist();
  }

  /** Removes an earlier conversation. The live one stays. */
  closeConversation(id: number) {
    if (id === this.cur.id) return;
    this.sessions = this.sessions.filter((x) => x.id !== id);
    if (this.viewId === id) this.viewId = this.cur.id;
    this.persist();
  }

  title(x: Session): string {
    const khz = x.freq ? (x.freq / 1000).toFixed(1) : '';
    return [khz, x.qso.call].filter(Boolean).join(' ') || t('tabs.empty');
  }

  private settleTimer: ReturnType<typeof setTimeout> | null = null;

  /**
   * The radio moved. A big jump starts a new conversation, but only when the frequency has stayed
   * put for new_tab_seconds (5 by default), so turning the knob over the band does not open a tab per step.
   * `now` skips the wait (the operator typed the frequency).
   */
  private frequencyMoved(hz: number, now = false) {
    const live = this.cur;
    if (!live || hz <= 0) return;
    if (this.settleTimer) {
      clearTimeout(this.settleTimer);
      this.settleTimer = null;
    }
    if (live.freq === 0) {
      live.freq = hz;
      return;
    }
    if (Math.abs(hz - live.freq) <= NEW_CONVERSATION_HZ) return;
    if (now) {
      this.newConversation(hz);
      return;
    }
    this.settleTimer = setTimeout(() => {
      this.settleTimer = null;
      const l = this.cur;
      if (l && Math.abs(hz - l.freq) > NEW_CONVERSATION_HZ) this.newConversation(hz);
    }, Math.min(30, Math.max(1, this.settings?.new_tab_seconds ?? 5)) * 1000);
  }

  private persistTimer: ReturnType<typeof setTimeout> | null = null;

  /** Saves the conversations on this computer, so they are still there after a restart. */
  persist() {
    if (this.demo || typeof localStorage === 'undefined') return;
    if (this.persistTimer) clearTimeout(this.persistTimer);
    this.persistTimer = setTimeout(() => {
      this.persistTimer = null;
      try {
        const keep = $state.snapshot(this.sessions).filter((x) => x.messages.length > 0);
        localStorage.setItem(STORE_KEY, JSON.stringify(keep));
      } catch {
        // No storage (private window, blocked): the app works without it.
      }
    }, 1500);
  }

  private restore(): Session[] {
    if (typeof localStorage === 'undefined') return [];
    try {
      const raw = localStorage.getItem(STORE_KEY);
      const list = raw ? (JSON.parse(raw) as Session[]) : [];
      return Array.isArray(list) ? list.filter((x) => x && Array.isArray(x.messages)).slice(-MAX_SESSIONS) : [];
    } catch {
      return [];
    }
  }

  /** Update state: idle, checking, none (newest), available, downloading, error. */
  updateState = $state<'idle' | 'checking' | 'none' | 'available' | 'downloading' | 'error'>('idle');
  updateInfo = $state<UpdateInfo | null>(null);
  updateProgress = $state(0);
  version = $state('');

  /** Looks for a newer version. `quiet` (at start) shows nothing when there is none or when it fails. */
  async checkUpdate(quiet = false) {
    if (!this.api || this.updateState === 'checking' || this.updateState === 'downloading') return;
    this.updateState = 'checking';
    try {
      const found = await this.api.checkUpdate();
      this.updateInfo = found;
      this.updateState = found ? 'available' : quiet ? 'idle' : 'none';
    } catch {
      this.updateState = quiet ? 'idle' : 'error';
    }
  }

  async installUpdate() {
    if (!this.api || this.updateState !== 'available') return;
    this.updateState = 'downloading';
    this.updateProgress = 0;
    try {
      await this.api.installUpdate((f) => (this.updateProgress = f));
    } catch {
      this.updateState = 'error';
    }
  }

  async init(api: Api) {
    this.api = api;
    const snap = await api.getSnapshot();
    this.apply(snap.settings);
    this.policy = snap.policy;
    this.status = snap.status;
    this.radioName = snap.radio_name;
    this.swrAlert = snap.swr_alert;
    this.audio = snap.audio ?? null;
    this.demo = api.demo;
    this.tail = null;
    this.waterfallRows = [];
    this.waterfallLive = false;
    this.scopeRows = [];
    this.scopeLive = false;
    this.lastDecodeAt = { rx: 0, paddle: 0 };
    const earlier = api.demo ? [] : this.restore();
    for (const e of earlier) e.touched = Date.now();
    this.nextId = Math.max(100, ...earlier.map((x) => x.id + 1), ...earlier.flatMap((x) => x.messages.map((m) => m.id + 1)));
    this.sessions = earlier;
    const first = this.makeSession(snap.status.frequency_hz);
    this.sessions.push(first);
    this.viewId = first.id;
    if (api.demo) for (const m of demoMessages) this.ingest(m.from, m.text, m.time);
    this.ready = true;
    this.wizardOpen = !api.demo && !snap.settings.first_run_done;
    this.resetEntry();
    this.testMode = false;
    void this.refreshLog();
    void this.refreshActivation();
    this.updateState = 'idle';
    this.updateInfo = null;
    api.appVersion().then((v) => (this.version = v)).catch(() => {});
    if (!api.demo) void this.checkUpdate(true);
    this.off?.();
    this.off = api.onEvents((ev) => this.onEvents(ev));
    if (this.sweepTimer) clearInterval(this.sweepTimer);
    this.sweepTimer = api.demo ? null : setInterval(() => this.sweepTabs(), SWEEP_MS);
  }

  private sweepTimer: ReturnType<typeof setInterval> | null = null;

  dispose() {
    if (this.sweepTimer) clearInterval(this.sweepTimer);
    this.sweepTimer = null;
    if (this.settleTimer) clearTimeout(this.settleTimer);
    this.settleTimer = null;
    this.off?.();
  }

  private apply(s: Settings) {
    this.settings = s;
    i18n.set(s.language);
    this.setTheme(s.theme);
  }

  private setTheme(theme: Settings['theme']) {
    if (typeof document === 'undefined') return;
    const dark =
      theme === 'dark' ||
      (theme === 'system' && typeof matchMedia !== 'undefined' && matchMedia('(prefers-color-scheme: dark)').matches);
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
  }

  private onEvents(events: CoreEvent[]) {
    for (const ev of events) {
      if (ev.type === 'policy') {
        this.policy = ev.policy;
        continue;
      }
      if (ev.type === 'decoder') {
        this.decoder = ev.stats;
        continue;
      }
      if (ev.type === 'decode') {
        this.decoded(ev.source, ev.text, ev.conf, ev.tentative);
        continue;
      }
      if (ev.type === 'waterfall') {
        this.waterfallRows.push({ bins: ev.bins, lo_hz: ev.lo_hz, hz_per_bin: ev.hz_per_bin });
        if (this.waterfallRows.length > 200) this.waterfallRows.splice(0, this.waterfallRows.length - 200);
        if (!this.waterfallLive) this.waterfallLive = true;
        continue;
      }
      if (ev.type === 'audio_level') {
        this.audioLevel = ev.dbfs;
        this.audioPeak = ev.peak ?? 0;
        continue;
      }
      if (ev.type === 'bandwidth') {
        this.bandwidthHz = ev.hz;
        continue;
      }
      if (ev.type === 'audio') {
        this.audio = ev.status;
        continue;
      }
      if (ev.type === 'swr_alert') {
        this.swrAlert = ev.alert;
        this.status.transmitting = false;
        this.queue = [];
        this.cancelPractice();
        continue;
      }
      if (ev.type === 'practice_done') {
        this.practiceDone(ev.ids);
        continue;
      }
      if (ev.type === 'swr_cleared') {
        this.swrAlert = null;
        continue;
      }
      const e = ev.event;
      switch (e.kind) {
        case 'connected': this.status.connected = e.value; break;
        case 'frequency':
          this.status.frequency_hz = e.value;
          this.frequencyMoved(e.value);
          break;
        case 'mode': this.status.mode = e.value; break;
        case 'tx':
          this.status.transmitting = e.value;
          if (!e.value) this.swr = null;
          break;
        case 's_meter': this.status.s_meter = e.value; break;
        case 'cw_speed': this.status.cw_wpm = e.value; break;
        case 'queue': this.queue = e.value; break;
        case 'swr': this.swr = e.value; break;
        case 'warning': this.warning = e.value; break;
        case 'paddle_text': this.paddle(e.value); break;
        case 'scope': this.scopeSweep(e.value); break;
      }
    }
  }

  /**
   * Decoded text from the audio decoder. Text that follows within a few seconds continues the
   * same bubble, a longer pause starts a new one. `conf` holds one confidence per character,
   * the interface fades the unsure ones. The tentative tail is shown faded after the bubble.
   */
  decoded(source: 'rx' | 'paddle', text: string, conf: number[], tentative: TentativeChar[], now = Date.now()) {
    const live = this.cur;
    if (text) {
      const last = live.messages[live.messages.length - 1];
      const goesOn = last && last.from === source && now - this.lastDecodeAt[source] < BUBBLE_PAUSE_MS;
      let target: Message | undefined;
      if (goesOn) {
        const before = last.conf ?? [...last.text].map(() => 1);
        last.text += text;
        last.conf = [...before, ...conf];
        target = last;
      } else {
        const clean = text.replace(/^\s+/, '');
        if (clean) {
          const cut = text.length - clean.length;
          live.messages.push({ id: this.nextId++, from: source, text: clean, conf: conf.slice(cut), time: new Date(now).toTimeString().slice(0, 5) });
          target = live.messages[live.messages.length - 1];
        }
      }
      this.lastDecodeAt[source] = now;
      this.touch(live, now);
      if (target && source === 'rx' && !this.hideRx) {
        const h = heardIn(target.text, this.settings?.station.calls ?? []);
        if (h.call) target.call = h.call;
        this.applyHeard(target.text, target);
      }
      this.persist();
    }
    this.tail = tentative.length ? { from: source, chars: tentative } : this.tail?.from === source ? null : this.tail;
  }

  /**
   * Text keyed with the paddle arrives in small pieces. Pieces that follow each other quickly
   * continue the same bubble, a pause starts a new one.
   */
  paddle(piece: string, now = Date.now()) {
    const live = this.cur;
    const last = live.messages[live.messages.length - 1];
    if (last && last.from === 'paddle' && now - this.lastPaddleAt < 6000) {
      last.text += piece;
    } else {
      this.ingest('paddle', piece);
    }
    this.lastPaddleAt = now;
    this.touch(live, now);
    this.persist();
  }

  private applySnapshot(snap: Snapshot) {
    this.status = snap.status;
    this.radioName = snap.radio_name;
    this.policy = snap.policy;
    this.queue = [];
  }

  async refreshPorts() {
    try {
      this.ports = await this.api.listPorts();
    } catch {
      this.ports = [];
    }
  }

  async connectRadio(radio: RadioSettings): Promise<boolean> {
    this.error = null;
    this.connecting = true;
    try {
      this.applySnapshot(await this.api.connect($state.snapshot(radio) as RadioSettings));
      if (this.settings) this.settings.radio = $state.snapshot(radio) as RadioSettings;
      return true;
    } catch (e) {
      this.fail(e);
      return false;
    } finally {
      this.connecting = false;
    }
  }

  async disconnectRadio() {
    try {
      this.applySnapshot(await this.api.disconnect());
    } catch (e) {
      this.fail(e);
    }
  }

  /** The operator confirms the SWR alert. */
  async ackSwr() {
    try {
      await this.api.ackSwr();
      this.swrAlert = null;
    } catch (e) {
      this.fail(e);
    }
  }

  clearWarning() {
    this.warning = null;
  }

  /** The station profile in use (home, portable). */
  get profile() {
    const s = this.settings;
    return s?.profiles.find((p) => p.id === s.profile) ?? null;
  }

  /** The call on the air: the first call of the operator plus the suffix of the profile (ON3VZ/P). */
  get myCall(): string {
    const base = this.settings?.station.calls[0] ?? '';
    return base ? `${base}${this.profile?.suffix ?? ''}`.toUpperCase() : '';
  }

  /** Variables for chip templates. */
  get vars(): Record<string, string> {
    const st = this.settings?.station;
    return {
      CALL: this.cur.qso.call,
      MYCALL: this.myCall,
      NAME: this.cur.qso.name,
      QTH: this.cur.qso.qth,
      RST: this.cur.qso.rst,
      // The serial number exists in contest mode only. Cut numbers when the profile asks for it.
      NR: this.settings?.contest_mode ? this.serialText : '',
      MYNAME: st?.name ?? '',
      MYQTH: st?.qth ?? '',
      // Own variables: reserved names are left out by ownVars, so they never replace the names above.
      ...ownVars(this.settings?.contest),
    };
  }

  /** The next serial number as it is sent: three digits, as cut numbers when the profile asks for it. */
  get serialText(): string {
    const nr = String(this.testMode ? this.testNr : (this.settings?.contest_nr ?? 1)).padStart(3, '0');
    return activeProfile(this.settings?.contest)?.cut_numbers ? cutNumber(nr) : nr;
  }

  private fail(e: unknown) {
    const code = codeOf(e);
    const key = `error.${code}`;
    let text = t(key) === key ? t('error.unknown') : t(key);
    // For a communication error the technical reason helps to find the cause (wrong port, port in use).
    const detail = e && typeof e === 'object' && 'message' in e ? String((e as CoreFailure).message ?? '') : '';
    if (code === 'io' && detail) text = `${text} (${detail})`;
    this.error = text;
  }

  clearError() {
    this.error = null;
  }

  async requestPolicy(p: TxPolicy) {
    this.error = null;
    if (p === this.policy) return;
    if (p === 'live') {
      this.confirmLive = true;
      return;
    }
    await this.applyPolicy(p, false);
  }

  async confirmLiveSwitch() {
    this.confirmLive = false;
    await this.applyPolicy('live', true);
  }

  private async applyPolicy(p: TxPolicy, confirmed: boolean) {
    try {
      this.cancelPractice();
      if (p !== 'practice' && this.practiceRun) await this.stopExercise();
      await this.api.setPolicy(p, confirmed);
      this.policy = p;
      if (this.settings) this.settings.last_policy = p;
    } catch (e) {
      this.fail(e);
    }
  }

  /** The operator picked a callsign by hand (click or typing). The decoder leaves it alone from now on. */
  selectCall(call: string) {
    this.qso = { ...this.qso, call: call.toUpperCase() };
    this.callLocked = true;
    this.noteCallChange();
  }

  /** Back to automatic: the next decoded station fills `{CALL}` again. */
  unlockCall() {
    this.callLocked = false;
    const last = [...this.messages].reverse().find((m) => m.from === 'rx');
    if (last && this.viewingLive) this.applyHeard(last.text, last);
  }

  /**
   * Adds text to the live conversation, whatever its source.
   * `rx` text also feeds the QSO panel, so `{CALL}`, `{NAME}` and `{QTH}` in chips are filled
   * from what the decoder heard, unless the operator locked the callsign.
   */
  ingest(from: Message['from'], text: string, time = clockNow()) {
    const call = from === 'rx' ? heardIn(text, this.settings?.station.calls ?? []).call : undefined;
    const m: Message = { id: this.nextId++, from, call, text, time };
    this.cur.messages.push(m);
    this.touch(this.cur);
    if (from === 'rx') {
      const before = this.qso.call;
      this.applyHeard(text, m);
      if (this.qso.call !== before) this.noteCallChange();
      // In contest mode, after our exchange, propose what the other station sent back.
      if (this.settings?.contest_mode && this.qso.call && this.exchangeSent) {
        const sug = suggestFrom(text, this.settings.contest, this.qso.call);
        if (Object.keys(sug).length) this.suggestion = { ...this.suggestion, ...sug };
      }
    }
  }

  /**
   * Fills the live QSO from what was heard. A different callsign than the one in the live
   * conversation starts a new conversation, and the bubble that carries it moves there.
   */
  private applyHeard(text: string, bubble?: Message) {
    const h = heardIn(text, this.settings?.station.calls ?? []);
    let live = this.cur;
    if (!live.callLocked && h.call && h.call !== live.qso.call) {
      if (live.qso.call) {
        const wasShowingLive = this.viewingLive;
        if (bubble) {
          const i = live.messages.indexOf(bubble);
          if (i >= 0) live.messages.splice(i, 1);
        }
        const next = this.makeSession(this.status.frequency_hz);
        this.sessions.push(next);
        while (this.sessions.length > MAX_SESSIONS) this.sessions.shift();
        if (wasShowingLive) this.viewId = next.id;
        if (bubble) next.messages.push(bubble);
        live = next;
        this.tail = null;
      }
      live.qso = { ...this.blankQso(), call: h.call };
    }
    // Name and QTH only fill empty fields, and only for the station that is in the panel.
    if (h.call && h.call === live.qso.call) {
      if (h.name && !live.qso.name) live.qso.name = h.name;
      if (h.qth && !live.qso.qth) live.qso.qth = h.qth;
    }
  }

  /** Sends the text of a chip or the typed text. In listen mode nothing is sent. */
  /** Adds a code or abbreviation to the send box, with a space before it when needed. */
  insertText(text: string) {
    const d = this.draft;
    this.draft = (d && !d.endsWith(' ') ? d + ' ' : d) + text.toUpperCase() + ' ';
  }

  async send(text: string) {
    this.error = null;
    this.notice = null;
    const clean = text.trim();
    if (!clean) return;
    this.viewId = this.cur.id;
    try {
      const out = await this.api.sendText(clean);
      const time = clockNow();
      if (out.kind === 'sent') {
        this.ingest('tx', clean, time);
      } else {
        this.ingest('practice', clean, time);
        this.notice = t('composer.simulated', { seconds: out.seconds.toFixed(1) });
        this.practiceBusy = true;
        this.stopSidetone = playSidetone(clean, this.status.cw_wpm);
        this.practiceTimer = setTimeout(() => {
          this.cancelPractice();
          // Your transmission is over: the simulated stations react to it.
          if (this.sim) this.simKey(this.sim.userSent(clean, Date.now() / 1000));
        }, out.seconds * 1000 + 100);
      }
      this.draft = '';
      this.persist();
    } catch (e) {
      this.fail(e);
    }
  }

  async sendChip(template: string) {
    const r = expand(template, this.vars);
    if (r.missing.length) {
      this.error = t('composer.missing', { vars: r.missing.join(', ') });
      return;
    }
    await this.send(r.text);
  }

  private cancelPractice() {
    this.stopSidetone?.();
    this.stopSidetone = null;
    if (this.practiceTimer) clearTimeout(this.practiceTimer);
    this.practiceTimer = null;
    this.practiceBusy = false;
  }

  async stop() {
    this.cancelPractice();
    try {
      await this.api.stopTx();
    } catch (e) {
      this.fail(e);
    }
  }

  async setSpeed(wpm: number) {
    try {
      await this.api.setSpeed(wpm);
      this.status.cw_wpm = wpm;
    } catch (e) {
      this.fail(e);
    }
  }

  async setFrequency(hz: number) {
    try {
      await this.api.setFrequency(hz);
      this.status.frequency_hz = hz;
      this.frequencyMoved(hz, true);
    } catch (e) {
      this.fail(e);
    }
  }

  async saveSettings(s: Settings) {
    try {
      await this.api.saveSettings($state.snapshot(s) as Settings);
      this.apply(s);
      this.notice = t('settings.saved');
    } catch (e) {
      this.fail(e);
    }
  }

  async refreshInputs() {
    try {
      this.inputs = await this.api.listAudioInputs();
    } catch {
      this.inputs = [];
    }
  }

  async startAudio() {
    this.error = null;
    try {
      this.audio = await this.api.startAudio();
    } catch (e) {
      this.fail(e);
      const detail = e && typeof e === 'object' && 'message' in e ? String((e as CoreFailure).message ?? '') : '';
      if (detail) this.error = `${this.error} (${detail})`;
    }
  }

  async stopAudio() {
    try {
      this.audio = await this.api.stopAudio();
      this.audioLevel = null;
    } catch (e) {
      this.fail(e);
    }
  }

  async decodeFile(path: string) {
    this.error = null;
    try {
      this.audio = await this.api.decodeFile(path.trim());
    } catch (e) {
      this.fail(e);
    }
  }

  private scopeSweep(s: ScopeSweep) {
    this.scopeOutOfRange = s.out_of_range;
    if (s.out_of_range || s.data.length === 0) return;
    this.scopeLo = s.lo_hz;
    this.scopeHi = s.hi_hz;
    this.scopeRows.push(s);
    if (this.scopeRows.length > 50) this.scopeRows.splice(0, this.scopeRows.length - 50);
    if (!this.scopeLive) this.scopeLive = true;
  }

  /** A click on the band scope: the radio tunes there (rounded to 10 Hz). Allowed in every mode. */
  async tuneTo(hz: number) {
    const r = Math.round(hz / 10) * 10;
    if (r > 0) await this.setFrequency(r);
  }

  /** Shows or hides the band scope or the audio waterfall, and remembers it. */
  async setView(part: 'band_scope' | 'audio_waterfall', on: boolean) {
    if (!this.settings) return;
    const next = { ...($state.snapshot(this.settings) as Settings), [part]: on };
    try {
      await this.api.saveSettings(next);
      this.settings = next;
      if (part === 'band_scope' && !on) this.scopeLive = false;
    } catch (e) {
      this.fail(e);
    }
  }

  /** The decoder listens to another pitch (a click on the waterfall). */
  private afcHold: ReturnType<typeof setTimeout> | null = null;

  /**
   * Sets the decoder tone. After a click in the waterfall the AFC waits 4 seconds, so it does not
   * pull the tone away from the spot the operator picked before the signal has settled.
   */
  async setTone(hz: number, fromClick = false) {
    const v = Math.round(Math.min(1500, Math.max(200, hz)));
    try {
      if (fromClick && this.settings?.audio.afc) {
        await this.api.setAfc(false);
        if (this.afcHold) clearTimeout(this.afcHold);
        this.afcHold = setTimeout(() => {
          this.afcHold = null;
          if (this.settings?.audio.afc) void this.api.setAfc(true).catch(() => {});
        }, 4000);
      }
      await this.api.setTone(v);
      if (this.audio) this.audio.tone_hz = v;
      if (this.settings) this.settings.audio.tone_hz = v;
    } catch (e) {
      this.fail(e);
    }
  }

  /** How strict the decoder is about signals in noise (1 to 8). Applies at once. */
  async setSquelch(value: number) {
    const v = Math.round(Math.min(8, Math.max(1, value)) * 10) / 10;
    try {
      await this.api.setSquelch(v);
      if (this.settings) this.settings.audio.squelch = v;
    } catch (e) {
      this.fail(e);
    }
  }

  /** The decoder looks for the strongest signal by itself (on) or stays where it was put. */
  async setAutoTone(on: boolean) {
    try {
      await this.api.setAutoTone(on);
      if (this.settings) this.settings.audio.auto_tone = on;
    } catch (e) {
      this.fail(e);
    }
  }

  /** Adds or replaces an own abbreviation in the code library. */
  async addUserWord(code: string, meaning: string) {
    const c = code.trim().toUpperCase().replace(/\s+/g, '');
    const m = meaning.trim();
    if (!this.settings || !c || !m) return;
    const rest = this.settings.user_words.filter((w) => w.code !== c);
    await this.patchSettings({ user_words: [...rest, { code: c, meaning: m }] });
  }

  async removeUserWord(code: string) {
    if (!this.settings) return;
    await this.patchSettings({ user_words: this.settings.user_words.filter((w) => w.code !== code) });
  }

  /** Sending is possible: the live conversation is on screen, not listening only, no SWR alert open. */
  get canSend(): boolean {
    return this.viewingLive && this.policy !== 'listen_only' && !(this.policy === 'live' && this.swrAlert);
  }

  /** A pile-up key (F1 to F4): sends its text when sending is possible. */
  async pressPileup(fkey: string) {
    const c = pileupChips.find((x) => x.fkey === fkey);
    if (c && this.canSend) await this.sendChip(c.template);
  }

  /** Saves a change of the settings without the "saved" notice (for switches on the main screen). */
  private async patchSettings(patch: Partial<Settings>) {
    if (!this.settings) return;
    const next = { ...($state.snapshot(this.settings) as Settings), ...patch };
    try {
      await this.api.saveSettings(next);
      this.settings = next;
    } catch (e) {
      this.fail(e);
    }
  }

  /** Gain, filter width and speed range of the decoder. Applies at once and is remembered. */
  async setTuning(change: Partial<Tuning>) {
    if (!this.settings) return;
    const a = this.settings.audio;
    const t: Tuning = {
      gain_db: Math.min(20, Math.max(-20, change.gain_db ?? a.gain_db)),
      wide_filter: change.wide_filter ?? a.wide_filter,
      min_wpm: change.min_wpm ?? a.min_wpm,
      max_wpm: change.max_wpm ?? a.max_wpm,
      lock_wpm: change.lock_wpm ?? a.lock_wpm,
    };
    t.min_wpm = Math.min(55, Math.max(5, Math.round(t.min_wpm)));
    t.max_wpm = Math.min(60, Math.max(t.min_wpm + 5, Math.round(t.max_wpm)));
    t.lock_wpm = t.lock_wpm >= 5 ? Math.min(60, Math.round(t.lock_wpm)) : 0;
    try {
      await this.api.setTuning(t);
      Object.assign(this.settings.audio, t);
    } catch (e) {
      this.fail(e);
    }
  }

  /** Weak, medium or strong: how strict the decoder is about signals in noise. */
  async setPreset(name: PresetName) {
    await this.setSquelch(PRESETS[name]);
  }

  /** The preset that matches the squelch value, or null for a hand-set value. */
  get preset(): PresetName | null {
    const v = this.settings?.audio.squelch;
    return (Object.keys(PRESETS) as PresetName[]).find((k) => Math.abs(PRESETS[k] - (v ?? 0)) < 0.05) ?? null;
  }

  async setAfc(on: boolean) {
    if (this.afcHold) {
      clearTimeout(this.afcHold);
      this.afcHold = null;
    }
    try {
      await this.api.setAfc(on);
      if (this.settings) this.settings.audio.afc = on;
    } catch (e) {
      this.fail(e);
    }
  }

  /** Once: the decoder moves to the strongest keyed tone heard lately. */
  async goStrongest() {
    try {
      await this.api.goStrongest();
    } catch (e) {
      this.fail(e);
    }
  }

  /** The signal report used in chips and new conversations: 5NN or 599. */
  async setDefaultRst(v: Settings['default_rst']) {
    await this.patchSettings({ default_rst: v });
  }

  /** Contest mode on: with a stored number the operator chooses to continue or restart. */
  async requestContest(on: boolean) {
    if (!this.settings) return;
    if (!on) {
      this.contestAsk = false;
      await this.patchSettings({ contest_mode: false });
      return;
    }
    if (this.settings.contest_nr > 1) this.contestAsk = true;
    else {
      await this.patchSettings({ contest_mode: true });
      this.resetEntry();
      void this.refreshLog();
    }
  }

  async answerContest(restart: boolean) {
    this.contestAsk = false;
    if (restart && this.settings) {
      // A new contest: the number starts again and earlier QSOs no longer count as dupes.
      const c = $state.snapshot(this.settings.contest) as Settings['contest'];
      await this.patchSettings({ contest_mode: true, contest_nr: 1, contest: { ...c, session: (c.session || 1) + 1 } });
    } else {
      await this.patchSettings({ contest_mode: true });
    }
    this.resetEntry();
    void this.refreshLog();
  }

  async setContestNr(n: number) {
    await this.patchSettings({ contest_nr: Math.min(9999, Math.max(1, Math.round(n) || 1)) });
  }



  // ---- Practice simulator ----

  /** The running exercise, or null. */
  practiceRun = $state<{ exercise: PracticeSettings['exercise']; startedTs: number; endsAt: number | null } | null>(null);
  /** The score of the last exercise, shown until the operator closes it. */
  practiceScore = $state<Score | null>(null);
  private sim: PracticeSim | null = null;
  private simIds = new Map<number, string>();
  private simTimer: ReturnType<typeof setInterval> | null = null;
  private practiceRestore: { contest_mode: boolean; testMode: boolean; audio: boolean } | null = null;

  /** Without decoder help the decoded text is hidden: train your ear. */
  get hideRx(): boolean {
    return !!this.practiceRun && !this.settings?.practice.decoder_help;
  }

  async setPracticeCfg(patch: Partial<PracticeSettings>) {
    if (!this.settings) return;
    await this.patchSettings({ practice: { ...($state.snapshot(this.settings.practice) as PracticeSettings), ...patch } });
    if (patch.volume !== undefined && this.practiceRun) {
      try {
        await this.api.practiceVolume(patch.volume);
      } catch {
        /* the volume is not important enough to bother the operator */
      }
    }
  }

  /** Starts an exercise: the decoder listens to the practice band, QSOs go to the test log. */
  async startExercise() {
    if (!this.settings || this.practiceRun) return;
    if (this.policy !== 'practice') {
      this.error = t('error.practice_mode_needed');
      return;
    }
    const cfg = $state.snapshot(this.settings.practice) as PracticeSettings;
    if (cfg.exercise === 'contest' && !this.settings.contest.active) {
      this.error = t('practice.need_profile');
      return;
    }
    this.error = null;
    this.practiceScore = null;
    this.practiceRestore = { contest_mode: this.settings.contest_mode, testMode: this.testMode, audio: !!this.audio?.running };
    const seed = Math.floor(Math.random() * 1e9) + 1;
    try {
      this.audio = await this.api.practiceStart(cfg.noise, cfg.volume, seed);
    } catch (e) {
      this.practiceRestore = null;
      this.fail(e);
      return;
    }
    const wantContest = cfg.exercise === 'contest';
    if (this.settings.contest_mode !== wantContest) await this.patchSettings({ contest_mode: wantContest });
    await this.setTestMode(true);
    if (wantContest) this.runMode = 'run';
    // A fresh conversation, so the exercise does not mix with what was on the screen.
    if (this.cur.messages.length) this.newConversation();
    const now = Date.now() / 1000;
    this.sim = new PracticeSim({ cfg, myCall: this.myCall, myName: this.settings.station.name.split(/\s+/)[0] ?? '', toneHz: this.settings.audio.tone_hz, contest: $state.snapshot(this.settings.contest) as Settings['contest'], seed });
    this.practiceRun = { exercise: cfg.exercise, startedTs: Math.floor(now), endsAt: wantContest && cfg.minutes > 0 ? now + cfg.minutes * 60 : null };
    this.simKey(this.sim.start(now));
    this.simTimer = setInterval(() => this.simTick(), 500);
  }

  private simTick() {
    if (!this.sim || !this.practiceRun) return;
    const now = Date.now() / 1000;
    if (this.practiceRun.endsAt && now >= this.practiceRun.endsAt) {
      void this.stopExercise();
      return;
    }
    // Nobody calls while you are sending.
    if (!this.practiceBusy) this.simKey(this.sim.tick(now));
  }

  private simKey(txs: SimTx[]) {
    for (const { call, ...signal } of txs) {
      this.api.practiceKey(signal).then(
        (id) => this.simIds.set(id, call),
        (e) => {
          // The station never got on the air: let the simulator know at once.
          this.sim?.done(call, Date.now() / 1000);
          this.fail(e);
        },
      );
    }
  }

  private practiceDone(ids: number[]) {
    for (const id of ids) {
      const call = this.simIds.get(id);
      this.simIds.delete(id);
      if (call && this.sim) this.simKey(this.sim.done(call, Date.now() / 1000));
    }
  }

  /** Ends the exercise, shows the score and gives the decoder back to the radio. */
  async stopExercise() {
    const run = this.practiceRun;
    const sim = this.sim;
    if (!run || !sim) return;
    if (this.simTimer) clearInterval(this.simTimer);
    this.simTimer = null;
    this.sim = null;
    this.practiceRun = null;
    this.simIds.clear();
    try {
      await this.api.practiceClear();
      await this.api.practiceStop();
    } catch (e) {
      this.fail(e);
    }
    let logged: LogQso[] = [];
    try {
      logged = (await this.api.logList({ test: true, from_ts: run.startedTs })).filter((q) => q.ts >= run.startedTs);
    } catch (e) {
      this.fail(e);
    }
    this.practiceScore = sim.score(logged, Date.now() / 1000);
    const r = this.practiceRestore;
    this.practiceRestore = null;
    if (r) {
      if (this.settings && this.settings.contest_mode !== r.contest_mode) await this.patchSettings({ contest_mode: r.contest_mode });
      await this.setTestMode(r.testMode);
      if (r.audio) await this.startAudio();
      else if (this.audio) this.audio = { ...this.audio, running: false };
    }
  }

  // ---- Station profiles and activations ----

  activation = $state<Activation | null>(null);
  actStats = $state<ActivationStats | null>(null);
  /** The other station's park or summit reference (park to park), for the next QSO. */
  theirRef = $state({ prog: 'WWFF', ref: '' });

  async refreshActivation() {
    if (!this.api) return;
    try {
      this.activation = await this.api.activationCurrent();
      this.actStats = this.activation ? await this.api.activationStats(this.activation.id) : null;
    } catch (e) {
      this.fail(e);
    }
  }

  /** The operator chooses another station. Never changes by itself, and not during an activation. */
  async setProfile(id: string) {
    if (this.activation || !this.settings?.profiles.some((p) => p.id === id)) return;
    await this.patchSettings({ profile: id });
  }

  /** Starts an activation. Returns an error code of the core for the form, or null when it worked. */
  async startActivation(a: Partial<Activation>): Promise<string | null> {
    try {
      this.activation = await this.api.activationStart({ id: 0, profile: '', grid: '', power: '', wwff: '', pota: '', sota: '', note: '', started_ts: 0, ended_ts: 0, ...a });
      await this.refreshActivation();
      return null;
    } catch (e) {
      return codeOf(e);
    }
  }

  async endActivation() {
    try {
      await this.api.activationEnd();
      this.activation = null;
      this.actStats = null;
    } catch (e) {
      this.fail(e);
    }
  }

  async setScopeSize(key: 'scope_size' | 'scope_size_contest', v: Settings['scope_size']) {
    await this.patchSettings({ [key]: v } as Partial<Settings>);
  }

  // ---- Logbook and contest entry ----

  /** Run (you call CQ) or S&P (you answer stations). */
  runMode = $state<RunMode>('run');
  /** Test mode: QSOs go to the separate test log and the number counts apart. */
  testMode = $state(false);
  /** Received report, serial number and extra field of the station being worked. */
  entry = $state({ rst: '', nr: '', extra: '' });
  /** Our exchange (or our call in S&P) was sent for the current call. */
  exchangeSent = $state(false);
  mycallSent = $state(false);
  /** The number that was sent but not logged yet, with the call it was sent to. */
  sentPending = $state<{ call: string; nr: string } | null>(null);
  /** Proposal from the decoder for the fields of the entry line. Grey until accepted. */
  suggestion = $state<Suggestion>({});
  dupeInfo = $state<DupeInfo | null>(null);
  /** The last QSOs of the log on screen (5), the count and the times of this contest run. */
  recent = $state<LogQso[]>([]);
  qsoCount = $state(0);
  qsoTimes = $state<number[]>([]);
  /** Sections and other extra values logged before, for completing while typing. */
  knownExtra = $state<string[]>([]);
  testNr = $state(1);
  exportOpen = $state(false);
  editing = $state<LogQso | null>(null);
  lastLogged = $state<LogQso | null>(null);

  /** The id written in the log: the profile, or `free` without one. Empty outside contest mode. */
  get contestId(): string {
    if (!this.settings?.contest_mode) return '';
    return this.contestProfile?.id ?? 'free';
  }
  get contestSession(): number {
    return this.settings?.contest.session ?? 1;
  }
  get band(): string {
    return bandOf(this.status.frequency_hz);
  }
  /** The messages behind F1 to F8 for the current mode. */
  get contestKeyList() {
    return contestKeys(this.runMode, this.settings?.contest);
  }
  get entryComplete(): boolean {
    return entryComplete(this.settings?.contest, this.qso.call, this.entry.nr, this.entry.extra);
  }
  /** The received extra field applies to this call (for example the section of a Belgian station). */
  get recvField() {
    const f = this.contestProfile?.recv ?? null;
    return f;
  }

  resetEntry() {
    this.entry = { rst: this.settings?.default_rst ?? '5NN', nr: '', extra: '' };
    this.exchangeSent = false;
    this.mycallSent = false;
    this.sentPending = null;
    this.suggestion = {};
    this.dupeInfo = null;
  }

  /** The call on the entry line changed (typed, clicked or heard): check dupes, warn about an unlogged number. */
  noteCallChange() {
    const call = this.qso.call.trim().toUpperCase();
    if (this.sentPending && this.sentPending.call !== call) {
      this.warning = t('contest.unlogged', { nr: this.sentPending.nr, call: this.sentPending.call });
      this.sentPending = null;
      this.exchangeSent = false;
      this.mycallSent = false;
      this.entry = { ...this.entry, nr: '', extra: '' };
      this.suggestion = {};
    }
    void this.refreshDupe();
  }

  async refreshDupe() {
    const call = this.qso.call.trim().toUpperCase();
    if (!call || !this.settings?.contest_mode) {
      this.dupeInfo = null;
      return;
    }
    try {
      this.dupeInfo = await this.api.logDupe(call, this.contestId, this.contestSession, this.testMode);
    } catch {
      this.dupeInfo = null;
    }
  }

  /** Loads the last QSOs, the count and the times for the rate. */
  async refreshLog() {
    if (!this.api) return;
    try {
      const contest = this.contestId;
      const all = await this.api.logList(contest ? { contest, session: this.contestSession, test: this.testMode } : { test: this.testMode, limit: 500 });
      this.recent = all.slice(0, 5);
      this.qsoCount = all.length;
      this.qsoTimes = all.map((q) => q.ts);
      this.knownExtra = [...new Set(all.map((q) => q.extra_rcvd.toUpperCase()).filter(Boolean))].sort();
      if (this.testMode) this.testNr = await this.api.logNextTestSerial();
    } catch (e) {
      this.fail(e);
    }
  }

  async setTestMode(on: boolean) {
    this.testMode = on;
    this.resetEntry();
    await this.refreshLog();
    await this.refreshDupe();
  }

  async clearTestLog() {
    try {
      await this.api.logClearTest();
      await this.refreshLog();
      this.notice = t('contest.test.cleared');
    } catch (e) {
      this.fail(e);
    }
  }

  setRunMode(m: RunMode) {
    this.runMode = m;
  }

  /** A function key in contest mode: sends its message and notes what has been sent. */
  async pressContestKey(key: string): Promise<boolean> {
    const k = this.contestKeyList.find((x) => x.key === key);
    if (!k) return false;
    if (this.canSend) {
      const r = expand(k.template, this.vars);
      if (r.missing.length) {
        this.error = t('composer.missing', { vars: r.missing.join(', ') });
        return true;
      }
      await this.send(r.text);
      if (this.error) return true;
      this.noteSent(key, k.template);
    }
    if (k.log) await this.logQso();
    return true;
  }

  private noteSent(key: string, template: string) {
    if (key === 'F2' && template.includes('{NR}') && this.serialUsed) this.sentPending = { call: this.qso.call.trim().toUpperCase(), nr: this.serialText };
    if (key === 'F2') this.exchangeSent = true;
    if (key === 'F1' && this.runMode === 'sp') this.mycallSent = true;
  }

  /** Enter does the next logical thing (N1MM calls it ESM). */
  async pressEnter() {
    const a = enterAction(this.runMode, { call: this.qso.call, exchangeSent: this.exchangeSent, mycallSent: this.mycallSent, complete: this.entryComplete });
    if (a.key) {
      const k = this.contestKeyList.find((x) => x.key === a.key);
      if (k && this.canSend) {
        const r = expand(k.template, this.vars);
        if (r.missing.length) {
          this.error = t('composer.missing', { vars: r.missing.join(', ') });
          return;
        }
        await this.send(r.text);
        if (this.error) return;
        this.noteSent(a.key, k.template);
      }
    }
    if (a.log) await this.logQso();
  }

  /** Puts the decoder's proposal in the empty fields. */
  acceptSuggestion() {
    const s = this.suggestion;
    this.entry = { rst: s.rst ?? this.entry.rst, nr: this.entry.nr || s.nr || '', extra: this.entry.extra || s.extra || '' };
    this.suggestion = {};
  }

  /** The operator tells where a word of the decoded text belongs. */
  assignWord(word: string, target: 'call' | 'rst' | 'nr' | 'extra' | 'name' | 'qth') {
    const w = word.toUpperCase().replace(/[^A-Z0-9/]/g, '');
    if (!w) return;
    if (target === 'call') this.selectCall(w);
    else if (target === 'rst') this.entry = { ...this.entry, rst: w };
    else if (target === 'nr') this.entry = { ...this.entry, nr: w.replace(/T/g, '0').replace(/N/g, '9') };
    else if (target === 'extra') this.entry = { ...this.entry, extra: w };
    else if (target === 'name') this.qso = { ...this.qso, name: w };
    else this.qso = { ...this.qso, qth: w };
  }

  /** Extra values of the sent exchange (everything except the call, report and number), for the log. */
  private extraSent(): string {
    const tpl = activeSent(this.settings?.contest);
    if (!tpl) return '';
    const names = [...tpl.matchAll(/\{([A-Z0-9_]+)\}/g)].map((m) => m[1]).filter((n) => !['CALL', 'RST', 'NR'].includes(n));
    return names.map((n) => this.vars[n] ?? '').filter(Boolean).join(' ');
  }

  /** Writes the QSO of the entry line to the log, raises the number and clears the line. */
  async logQso(): Promise<boolean> {
    if (!this.settings) return false;
    const call = this.qso.call.trim().toUpperCase();
    if (!call) {
      this.error = t('log.nocall');
      return false;
    }
    const contest = this.settings.contest_mode;
    const useNr = contest && this.serialUsed;
    const nrSent = useNr ? (this.testMode ? this.testNr : this.settings.contest_nr) : 0;
    const q: LogQso = {
      ...emptyLogQso(),
      call, band: this.band, freq_hz: this.status.frequency_hz, mode: 'CW',
      rst_sent: this.qso.rst, rst_rcvd: this.entry.rst || this.settings.default_rst,
      nr_sent: nrSent, nr_rcvd: this.entry.nr.trim(),
      extra_sent: contest ? this.extraSent() : '', extra_rcvd: this.entry.extra.trim().toUpperCase(),
      name: this.qso.name, qth: this.qso.qth, comment: '',
      contest: this.contestId, session: this.contestSession, my_call: this.myCall, test: this.testMode,
      their_prog: this.theirRef.ref.trim() ? this.theirRef.prog : '', their_ref: this.theirRef.ref.trim().toUpperCase(),
    };
    try {
      this.lastLogged = await this.api.logAdd(q);
    } catch (e) {
      this.fail(e);
      return false;
    }
    this.cur.logged = true;
    void this.refreshActivation();
    this.sentPending = null;
    this.theirRef = { prog: this.theirRef.prog, ref: '' };
    if (useNr) {
      if (this.testMode) this.testNr = nextSerial(this.testNr);
      else await this.setContestNr(nextSerial(this.settings.contest_nr));
    }
    this.notice = t('log.saved', { call });
    this.qso = { ...this.qso, call: '', name: '', qth: '' };
    this.callLocked = false;
    this.resetEntry();
    await this.refreshLog();
    this.persist();
    return true;
  }

  clearEntry() {
    this.qso = { ...this.qso, call: '', name: '', qth: '' };
    this.callLocked = false;
    this.resetEntry();
  }

  async saveEdit(q: LogQso) {
    try {
      await this.api.logUpdate(q);
      this.editing = null;
      await this.refreshLog();
    } catch (e) {
      this.fail(e);
    }
  }

  async deleteQso(id: number) {
    try {
      await this.api.logDelete(id);
      await this.refreshLog();
    } catch (e) {
      this.fail(e);
    }
  }

  async checkLog(filter: import('./types').LogFilter) {
    return this.api.logCheck(filter);
  }
  async exportLog(filter: import('./types').LogFilter, contestId: string) {
    return this.api.logExport(filter, contestId);
  }

  // ---- Contest profiles ----

  /** The note of a profile that was just chosen. Shown once, the operator closes it. */
  contestNote = $state<string | null>(null);

  get contestProfile() {
    return activeProfile(this.settings?.contest);
  }

  /** Chips of the active profile: its calls and the exchange. Without a profile only the plain exchange. */
  get contestChipList(): { id: string; label: string; template: string }[] {
    const c = this.settings?.contest;
    const p = activeProfile(c);
    if (!p) return contestChips.map((x) => ({ id: x.id, label: t(x.key), template: x.template }));
    const out = p.calls.map((call, i) => ({ id: `call${i}`, label: call, template: `${call} {MYCALL}` }));
    const tpl = activeSent(c);
    if (tpl) out.push({ id: 'exchange', label: t('chip.exchange'), template: tpl });
    return out;
  }

  /** Whether the active exchange uses {NR}: then the serial number is shown and counted. */
  get serialUsed(): boolean {
    return usesSerial(this.settings?.contest);
  }

  private async patchContest(patch: Partial<Settings['contest']>) {
    if (!this.settings) return;
    await this.patchSettings({ contest: { ...($state.snapshot(this.settings.contest) as Settings['contest']), ...($state.snapshot(patch) as Partial<Settings['contest']>) } });
  }

  async selectContestProfile(id: string) {
    if (!this.settings) return;
    await this.patchContest({ active: id, sent_index: 0 });
    const p = activeProfile(this.settings.contest);
    this.contestNote = p && p.note ? plain(p.note, (k) => t(k)) : null;
    // The rules note should not take screen space for long during a contest.
    if (this.contestNote) {
      const shown = this.contestNote;
      setTimeout(() => { if (this.contestNote === shown) this.contestNote = null; }, 12000);
    }
  }

  async setContestSent(i: number) {
    await this.patchContest({ sent_index: Math.max(0, Math.round(i)) });
  }

  /** Saves an own profile: new, or a change of an existing own profile. */
  async saveContestProfile(p: Settings['contest']['profiles'][number]) {
    if (!this.settings || isBuiltin(p.id)) return;
    const list = [...this.settings.contest.profiles];
    const i = list.findIndex((x) => x.id === p.id);
    if (i >= 0) list[i] = p;
    else list.push(p);
    await this.patchContest({ profiles: list });
  }

  async deleteContestProfile(id: string) {
    if (!this.settings || isBuiltin(id)) return;
    const c = this.settings.contest;
    await this.patchContest({
      profiles: c.profiles.filter((x) => x.id !== id),
      ...(c.active === id ? { active: '', sent_index: 0 } : {}),
    });
  }

  /** Makes an own copy of any profile and returns its id. */
  async copyContestProfile(id: string, name: string): Promise<string | null> {
    if (!this.settings) return null;
    const src = allProfiles(this.settings.contest).find((x) => x.id === id);
    if (!src) return null;
    const copy = copyContestProfileData(src, name, allProfiles(this.settings.contest), (k) => t(k));
    await this.patchContest({ profiles: [...this.settings.contest.profiles, copy] });
    return copy.id;
  }

  /** A new empty own profile. Returns its id. */
  async newContestProfile(name: string): Promise<string | null> {
    if (!this.settings) return null;
    const id = newProfileId(allProfiles(this.settings.contest));
    const p = { id, name, note: '', calls: ['CQ TEST'], sent: [{ label: t('contest.sent.default'), template: '{CALL} {RST} {NR}' }], recv: null, cut_numbers: false };
    await this.patchContest({ profiles: [...this.settings.contest.profiles, p] });
    return id;
  }

  async saveContestVars(vars: Settings['contest']['vars']) {
    await this.patchContest({ vars });
  }

  /** Reads an export file and merges it. Returns how many profiles came in and how many items were skipped. */
  async importContest(text: string, replaceSameName: boolean): Promise<{ added: number; skipped: number } | null> {
    if (!this.settings) return null;
    try {
      const imp = parseImport(text);
      await this.patchSettings({ contest: mergeImport($state.snapshot(this.settings.contest) as Settings['contest'], imp, replaceSameName) });
      return { added: imp.profiles.length, skipped: imp.skipped };
    } catch (e) {
      this.error = t(e instanceof Error && e.message === 'not_json' ? 'contest.import.notjson' : 'contest.import.notexport');
      return null;
    }
  }

  /** True while the decoder judges that a clear signal is present. */
  get signalPresent(): boolean {
    return this.decoder?.snr_db != null;
  }

  async toggleRecording() {
    this.error = null;
    try {
      if (this.audio?.recording) {
        await this.api.stopRecording();
        if (this.audio) this.audio.recording = false;
        this.notice = t('rec.saved', { path: this.audio?.record_path ?? '' });
      } else {
        const path = await this.api.startRecording();
        if (this.audio) {
          this.audio.recording = true;
          this.audio.record_seconds = 0;
          this.audio.record_path = path;
        }
      }
    } catch (e) {
      this.fail(e);
    }
  }

  async toggleSignals() {
    if (!this.settings) return;
    const s = { ...$state.snapshot(this.settings), signals_panel: !this.settings.signals_panel } as Settings;
    await this.saveSettings(s);
    this.notice = null;
  }

  estimate(text: string): number {
    return durationSeconds(text, this.status.cw_wpm);
  }
}

export const app = new AppState();
