import type { Api, AudioStatus, CoreEvent, CoreFailure, DecoderStats, InputDevice, PortInfo, RadioSettings, RadioStatus, Settings, Snapshot, SwrAlert, TentativeChar, TxPolicy } from './types';
import { i18n, t } from './i18n.svelte';
import { demoMessages, demoSignals, type Message, type Signal } from './demo';
import { expand } from './template';
import { heardIn } from './qso';
import { durationSeconds, playSidetone } from './morse';

export interface Qso {
  call: string;
  name: string;
  rst: string;
  qth: string;
}

const emptyQso = (): Qso => ({ call: '', name: '', rst: '599', qth: '' });

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
  messages = $state<Message[]>([]);
  signals = $state<Signal[]>(demoSignals);
  qso = $state<Qso>(emptyQso());
  /** True when the operator typed or clicked the callsign. Then the decoder stops overwriting it. */
  callLocked = $state(false);
  draft = $state('');
  error = $state<string | null>(null);
  notice = $state<string | null>(null);
  confirmLive = $state(false);
  settingsOpen = $state(false);
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
    this.messages = [];
    this.tail = null;
    this.waterfallRows = [];
    this.waterfallLive = false;
    this.lastDecodeAt = { rx: 0, paddle: 0 };
    this.qso = emptyQso();
    this.callLocked = false;
    if (api.demo) for (const m of demoMessages) this.ingest(m.from, m.text, m.time);
    this.ready = true;
    this.off?.();
    this.off = api.onEvents((ev) => this.onEvents(ev));
  }

  dispose() {
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
      if (ev.type === 'swr_cleared') {
        this.swrAlert = null;
        continue;
      }
      const e = ev.event;
      switch (e.kind) {
        case 'connected': this.status.connected = e.value; break;
        case 'frequency': this.status.frequency_hz = e.value; break;
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
      }
    }
  }

  /**
   * Decoded text from the audio decoder. Text that follows within a few seconds continues the
   * same bubble, a longer pause starts a new one. `conf` holds one confidence per character,
   * the interface fades the unsure ones. The tentative tail is shown faded after the bubble.
   */
  decoded(source: 'rx' | 'paddle', text: string, conf: number[], tentative: TentativeChar[], now = Date.now()) {
    if (text) {
      const last = this.messages[this.messages.length - 1];
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
          this.messages.push({ id: this.nextId++, from: source, text: clean, conf: conf.slice(cut), time: new Date(now).toTimeString().slice(0, 5) });
          target = this.messages[this.messages.length - 1];
        }
      }
      this.lastDecodeAt[source] = now;
      if (target && source === 'rx') {
        const h = heardIn(target.text, this.settings?.station.calls ?? []);
        if (h.call) target.call = h.call;
        this.applyHeard(target.text);
      }
    }
    this.tail = tentative.length ? { from: source, chars: tentative } : this.tail?.from === source ? null : this.tail;
  }

  /**
   * Text keyed with the paddle arrives in small pieces. Pieces that follow each other quickly
   * continue the same bubble, a pause starts a new one.
   */
  paddle(piece: string, now = Date.now()) {
    const last = this.messages[this.messages.length - 1];
    if (last && last.from === 'paddle' && now - this.lastPaddleAt < 6000) {
      last.text += piece;
    } else {
      this.ingest('paddle', piece);
    }
    this.lastPaddleAt = now;
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

  get myCall(): string {
    return this.settings?.station.calls[0] ?? '';
  }

  /** Variables for chip templates. */
  get vars() {
    const st = this.settings?.station;
    return {
      CALL: this.qso.call,
      MYCALL: this.myCall,
      NAME: this.qso.name,
      QTH: this.qso.qth,
      RST: this.qso.rst,
      MYNAME: st?.name ?? '',
      MYQTH: st?.qth ?? '',
    };
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
  }

  /** Back to automatic: the next decoded station fills `{CALL}` again. */
  unlockCall() {
    this.callLocked = false;
    const last = [...this.messages].reverse().find((m) => m.from === 'rx');
    if (last) this.applyHeard(last.text);
  }

  /**
   * Adds text to the conversation, whatever its source.
   * `rx` text also feeds the QSO panel, so `{CALL}`, `{NAME}` and `{QTH}` in chips are filled
   * from what the decoder heard, unless the operator locked the callsign.
   */
  ingest(from: Message['from'], text: string, time = new Date().toTimeString().slice(0, 5)) {
    const call = from === 'rx' ? heardIn(text, this.settings?.station.calls ?? []).call : undefined;
    this.messages.push({ id: this.nextId++, from, call, text, time });
    if (from === 'rx') this.applyHeard(text);
  }

  private applyHeard(text: string) {
    const h = heardIn(text, this.settings?.station.calls ?? []);
    if (!this.callLocked && h.call) {
      if (h.call !== this.qso.call) this.qso = { ...emptyQso(), call: h.call };
    }
    // Name and QTH only fill empty fields, and only for the station that is in the panel.
    if (h.call && h.call === this.qso.call) {
      if (h.name && !this.qso.name) this.qso.name = h.name;
      if (h.qth && !this.qso.qth) this.qso.qth = h.qth;
    }
  }

  /** Sends the text of a chip or the typed text. In listen mode nothing is sent. */
  async send(text: string) {
    this.error = null;
    this.notice = null;
    const clean = text.trim();
    if (!clean) return;
    try {
      const out = await this.api.sendText(clean);
      const time = new Date().toTimeString().slice(0, 5);
      if (out.kind === 'sent') {
        this.ingest('tx', clean, time);
      } else {
        this.ingest('practice', clean, time);
        this.notice = t('composer.simulated', { seconds: out.seconds.toFixed(1) });
        this.practiceBusy = true;
        this.stopSidetone = playSidetone(clean, this.status.cw_wpm);
        this.practiceTimer = setTimeout(() => this.cancelPractice(), out.seconds * 1000 + 100);
      }
      this.draft = '';
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

  /** The decoder listens to another pitch (a click on the waterfall). */
  async setTone(hz: number) {
    const v = Math.round(Math.min(1500, Math.max(200, hz)));
    try {
      await this.api.setTone(v);
      if (this.audio) this.audio.tone_hz = v;
      if (this.settings) this.settings.audio.tone_hz = v;
    } catch (e) {
      this.fail(e);
    }
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
