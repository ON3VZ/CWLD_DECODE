import type { Activation, ActivationStats, Api, AudioStatus, CheckIssue, ClusterProbe, ClusterSpot, ClusterStatus, DupeInfo, ExportResult, LogFilter, LogQso, CoreEvent, InputDevice, LinkReport, LookupInfo, LookupStatus, PortInfo, PracticeSignal, RadioSettings, SendOutcome, Settings, Snapshot, TxPolicy } from './types';

type Invoke = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

/** Real implementation: talks to the Rust core through Tauri commands. */
export async function createTauriApi(): Promise<Api> {
  const core = await import('@tauri-apps/api/core');
  const events = await import('@tauri-apps/api/event');
  const invoke = core.invoke as Invoke;
  let pending: { downloadAndInstall: (cb: (e: { event: string; data?: { contentLength?: number; chunkLength?: number } }) => void) => Promise<void> } | null = null;
  return {
    appVersion: async () => (await import('@tauri-apps/api/app')).getVersion(),
    async checkUpdate() {
      const { check } = await import('@tauri-apps/plugin-updater');
      const found = await check();
      pending = found;
      return found ? { version: found.version, notes: found.body ?? '' } : null;
    },
    async installUpdate(onProgress) {
      if (!pending) throw new Error('no update');
      // Let go of the radio port and the sound card first, so the installer finds nothing running.
      await invoke<void>('prepare_update');
      let total = 0;
      let done = 0;
      await pending.downloadAndInstall((e) => {
        if (e.event === 'Started') total = e.data?.contentLength ?? 0;
        else if (e.event === 'Progress') {
          done += e.data?.chunkLength ?? 0;
          if (total > 0) onProgress(Math.min(1, done / total));
        }
      });
      onProgress(1);
      await (await import('@tauri-apps/plugin-process')).relaunch();
    },
    getSnapshot: () => invoke<Snapshot>('get_snapshot'),
    setPolicy: (policy: TxPolicy, confirmed: boolean) => invoke<void>('set_policy', { policy, confirmed }),
    sendText: (text: string) => invoke<SendOutcome>('send_text', { text }),
    stopTx: () => invoke<void>('stop_tx'),
    setSpeed: (wpm: number) => invoke<void>('set_speed', { wpm }),
    setFrequency: (hz: number) => invoke<void>('set_frequency', { hz }),
    saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),
    listPorts: () => invoke<PortInfo[]>('list_ports'),
    connect: (radio: RadioSettings) => invoke<Snapshot>('connect', { radio }),
    disconnect: () => invoke<Snapshot>('disconnect'),
    ackSwr: async () => {
      await invoke<unknown>('ack_swr');
    },
    listAudioInputs: () => invoke<InputDevice[]>('list_audio_inputs'),
    startAudio: () => invoke<AudioStatus>('start_audio'),
    stopAudio: () => invoke<AudioStatus>('stop_audio'),
    decodeFile: (path: string) => invoke<AudioStatus>('decode_file', { path }),
    setTone: (hz: number) => invoke<void>('set_tone', { hz }),
    setSquelch: (value: number) => invoke<void>('set_squelch', { value }),
    setAutoTone: (on: boolean) => invoke<void>('set_auto_tone', { on }),
    setTuning: (t) => invoke<void>('set_tuning', { gainDb: t.gain_db, wideFilter: t.wide_filter, minWpm: t.min_wpm, maxWpm: t.max_wpm, lockWpm: t.lock_wpm }),
    setAfc: (on: boolean) => invoke<void>('set_afc', { on }),
    setFading: (on: boolean) => invoke<void>('set_fading', { on }),
    goStrongest: () => invoke<void>('go_strongest'),
    startRecording: () => invoke<string>('start_recording'),
    stopRecording: () => invoke<void>('stop_recording'),
    logAdd: (qso: LogQso) => invoke<LogQso>('log_add', { qso }),
    logList: (filter: LogFilter) => invoke<LogQso[]>('log_list', { filter }),
    logUpdate: (qso: LogQso) => invoke<void>('log_update', { qso }),
    logDelete: (id: number) => invoke<void>('log_delete', { id }),
    logClearTest: () => invoke<number>('log_clear_test'),
    logNextTestSerial: () => invoke<number>('log_next_test_serial'),
    logDupe: (call: string, contest: string, session: number, test: boolean) => invoke<DupeInfo>('log_dupe', { call, contest, session, test }),
    logExport: (filter: LogFilter, contestId: string) => invoke<ExportResult>('log_export', { filter, contestId }),
    linkResendLast: () => invoke<LinkReport>('link_resend_last'),
    lookupStatus: () => invoke<LookupStatus>('lookup_status'),
    lookupSetPassword: (password) => invoke<void>('lookup_set_password', { password }),
    lookupClearPassword: () => invoke<void>('lookup_clear_password'),
    lookupTest: () => invoke<{ subscriber: boolean }>('lookup_test'),
    lookupCall: (call) => invoke<LookupInfo | null>('lookup_call', { call }),
    logCheck: (filter: LogFilter) => invoke<CheckIssue[]>('log_check', { filter }),
    practiceStart: (noise: boolean, volume: number, seed: number) => invoke<AudioStatus>('practice_start', { noise, volume, seed }),
    practiceStop: () => invoke<void>('practice_stop'),
    practiceKey: (signal: PracticeSignal) => invoke<number>('practice_key', { signal }),
    practiceClear: () => invoke<void>('practice_clear'),
    practiceVolume: (volume: number) => invoke<void>('practice_volume', { volume }),
    keyingCapture: (on: boolean) => invoke<void>('keying_capture', { on }),
    activationStart: (activation: Activation) => invoke<Activation>('activation_start', { activation }),
    activationEnd: () => invoke<void>('activation_end'),
    activationCurrent: () => invoke<Activation | null>('activation_current'),
    activationStats: (id: number) => invoke<ActivationStats>('activation_stats', { id }),
    clusterStart: () => invoke<ClusterStatus>('cluster_start'),
    clusterStop: () => invoke<ClusterStatus>('cluster_stop'),
    clusterStatus: () => invoke<ClusterStatus>('cluster_status'),
    clusterSend: (line: string) => invoke<void>('cluster_send', { line }),
    clusterSpots: () => invoke<ClusterSpot[]>('cluster_spots'),
    clusterProbe: () => invoke<ClusterProbe[]>('cluster_probe'),
    demo: false,
    onEvents(handler) {
      let off: (() => void) | null = null;
      let cancelled = false;
      events.listen<CoreEvent[]>('core-events', (e) => handler(e.payload)).then((fn) => {
        if (cancelled) fn();
        else off = fn;
      });
      return () => {
        cancelled = true;
        off?.();
      };
    },
  };
}
