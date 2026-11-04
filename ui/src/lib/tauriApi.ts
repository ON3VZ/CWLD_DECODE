import type { Api, AudioStatus, CoreEvent, InputDevice, PortInfo, RadioSettings, SendOutcome, Settings, Snapshot, TxPolicy } from './types';

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
    startRecording: () => invoke<string>('start_recording'),
    stopRecording: () => invoke<void>('stop_recording'),
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
