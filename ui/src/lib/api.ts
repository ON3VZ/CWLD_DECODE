import type { Api } from './types';
import { MockApi } from './mockApi';

export function inTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function createApi(): Promise<Api> {
  if (inTauri()) {
    const { createTauriApi } = await import('./tauriApi');
    return createTauriApi();
  }
  const mock = new MockApi();
  // `?demo=1` hands the pretend radio to the page, so screenshots and demos can set the SWR.
  if (typeof location !== 'undefined' && new URLSearchParams(location.search).has('demo')) {
    (window as unknown as { __cwldMock: MockApi }).__cwldMock = mock;
  }
  return mock;
}
