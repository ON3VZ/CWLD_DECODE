import type { Api, AudioStatus, CoreEvent, InputDevice, PortInfo, RadioSettings, SendOutcome, Settings, Snapshot, TxPolicy } from './types';

type Invoke = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

/** Real implementation: talks to the Rust core through Tauri commands. */
export async function createTauriApi(): Promise<Api> {
  const core = await import('@tauri-apps/api/core');
  const events = await import('@tauri-apps/api/event');
  const invoke = core.invoke as Invoke;
  return {
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
