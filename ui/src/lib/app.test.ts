import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { app, SETTLE_MS } from './app.svelte';
import { MockApi, defaultSettings } from './mockApi';

async function fresh(calls: string[] = ['ON3VZ'], extra: Record<string, string> = {}) {
  const api = new MockApi({ station: { calls, name: 'KRISTOF', qth: 'HOBOKEN', locator: 'JO20EE', ...extra } });
  await app.init(api);
  return api;
}

describe('decoded callsign becomes {CALL}', () => {
  beforeEach(async () => {
    await fresh();
  });

  it('is filled from the decoded demo conversation, with name and QTH', () => {
    expect(app.qso.call).toBe('DL1ABC');
    expect(app.qso.name).toBe('HANS');
    expect(app.qso.qth).toBe('BERLIN');
    expect(app.vars.CALL).toBe('DL1ABC');
  });

  it('follows a new decoded station and clears the old name and QTH', () => {
    app.ingest('rx', 'CQ CQ DE F4XYZ F4XYZ K');
    expect(app.qso.call).toBe('F4XYZ');
    expect(app.qso.name).toBe('');
    expect(app.qso.qth).toBe('');
  });

  it('does not overwrite a callsign the operator fixed', () => {
    app.selectCall('G3ABC');
    app.ingest('rx', 'CQ DE F4XYZ K');
    expect(app.qso.call).toBe('G3ABC');
    app.unlockCall();
    expect(app.qso.call).toBe('F4XYZ');
  });

  it('never takes our own callsign', () => {
    app.ingest('rx', 'CQ DE ON3VZ K');
    expect(app.qso.call).toBe('DL1ABC');
  });

  it('typed and paddle text do not change the QSO panel', () => {
    app.ingest('tx', 'F4XYZ DE ON3VZ K');
    app.ingest('paddle', 'F4XYZ DE ON3VZ K');
    expect(app.qso.call).toBe('DL1ABC');
  });

  it('a chip with {CALL} gets the decoded callsign and uses my own name for {MYNAME}', async () => {
    const { expand } = await import('./template');
    const r = expand('{CALL} DE {MYCALL} NAME {MYNAME} QTH {MYQTH} K', app.vars);
    expect(r.missing).toEqual([]);
    expect(r.text).toBe('DL1ABC DE ON3VZ NAME KRISTOF QTH HOBOKEN K');
  });
});

describe('typed and paddle text share one conversation', () => {
  it('keeps the order and tells the sources apart', async () => {
    await fresh();
    const before = app.messages.length;
    app.ingest('paddle', 'AGN');
    app.ingest('tx', 'AGN PSE');
    const added = app.messages.slice(before).map((m) => m.from);
    expect(added).toEqual(['paddle', 'tx']);
    const froms = new Set(app.messages.map((m) => m.from));
    expect(froms.has('paddle') && froms.has('tx') && froms.has('rx')).toBe(true);
  });
});

describe('paddle text and radio events', () => {
  it('pieces that follow each other continue one bubble, a pause starts a new one', async () => {
    await fresh();
    const before = app.messages.length;
    app.paddle('R R', 100_000);
    app.paddle(' TNX', 101_000);
    expect(app.messages.length).toBe(before + 1);
    expect(app.messages[app.messages.length - 1]).toMatchObject({ from: 'paddle', text: 'R R TNX' });
    app.paddle('AGN', 200_000);
    expect(app.messages.length).toBe(before + 2);
  });

  it('typed text between two paddle pieces separates the bubbles', async () => {
    await fresh();
    app.paddle('QRZ', 300_000);
    app.ingest('tx', 'K');
    app.paddle('AGN', 301_000);
    const froms = app.messages.slice(-3).map((m) => m.from);
    expect(froms).toEqual(['paddle', 'tx', 'paddle']);
  });

  it('shows the queue and warnings from the radio layer', async () => {
    const api = await fresh();
    api.paddle('AR');
    await new Promise((r) => setTimeout(r, 600));
    expect(app.messages[app.messages.length - 1].from).toBe('paddle');
    app.dispose();
  });
});

describe('connecting a radio', () => {
  it('needs a port for a real radio and keeps the old state when it fails', async () => {
    await fresh();
    const ok = await app.connectRadio({ kind: 'icom', port: '', civ_address: 0xb6, baud: 115200, pipeline: false, auto_connect: true });
    expect(ok).toBe(false);
    expect(app.error).toMatch(/port|poort/i);
    expect(app.status.connected).toBe(true);
  });

  it('connects and disconnects', async () => {
    await fresh();
    expect(await app.connectRadio({ kind: 'qmx', port: 'COM6', civ_address: 0xb6, baud: 115200, pipeline: false, auto_connect: true })).toBe(true);
    expect(app.radioName).toContain('QMX');
    await app.disconnectRadio();
    expect(app.status.connected).toBe(false);
  });

  it('lists ports with a hint for an Icom', async () => {
    await fresh();
    await app.refreshPorts();
    expect(app.ports.length).toBeGreaterThan(0);
    expect(app.ports[0].hint).toBe('icom');
  });
});

describe('sending in live mode', () => {
  it('refuses a text that takes more than a minute and shows why', async () => {
    await fresh();
    await app.requestPolicy('live');
    await app.confirmLiveSwitch();
    await app.send('CQ '.repeat(200));
    expect(app.error).toMatch(/minut/i);
  });

  it('refuses a word longer than a block', async () => {
    await fresh();
    await app.requestPolicy('live');
    await app.confirmLiveSwitch();
    await app.send('A'.repeat(40));
    expect(app.error).toMatch(/word|woord/i);
  });
});

describe('SWR protection', () => {
  async function live() {
    const api = await fresh();
    await app.requestPolicy('live');
    await app.confirmLiveSwitch();
    return api;
  }
  const tick = () => new Promise((r) => setTimeout(r, 0));

  it('is on at 1.5 by default and QRX is on', async () => {
    await fresh();
    expect(app.settings?.swr).toEqual({ enabled: true, limit: 1.5, qrx_after_stop: true });
  });

  it('a good antenna shows the reading and no alert', async () => {
    const api = await live();
    api.setSwr(1.2);
    await app.send('CQ CQ CQ DE ON3VZ K');
    api.flush();
    api.emitSwr();
    api.emitSwr();
    api.emitSwr();
    await tick();
    expect(app.swr).toBeCloseTo(1.2);
    expect(app.swrAlert).toBeNull();
    expect(app.status.transmitting).toBe(true);
  });

  it('stops, shows an alert and blocks sending until the operator confirms', async () => {
    const api = await live();
    api.setSwr(2.8);
    await app.send('CQ CQ CQ DE ON3VZ K');
    api.flush();
    expect(app.status.transmitting).toBe(true);
    api.emitSwr();
    await tick();
    expect(app.swrAlert).toBeNull(); // one reading is not enough
    api.emitSwr();
    await tick();
    expect(app.swrAlert).toMatchObject({ limit: 1.5 });
    expect(app.status.transmitting).toBe(false);
    expect(app.queue).toEqual([]);
    await app.send('CQ');
    expect(app.error).toMatch(/SWR/i);
    await app.ackSwr();
    expect(app.swrAlert).toBeNull();
    api.setSwr(1.2);
    await app.send('CQ');
    expect(app.error).toBeNull();
  });

  it('after a restart the open alert is still there', async () => {
    const api = await live();
    api.setSwr(3);
    await app.send('CQ CQ CQ DE ON3VZ K');
    api.flush();
    api.emitSwr();
    api.emitSwr();
    await app.init(api);
    expect(app.swrAlert?.limit).toBe(1.5);
  });

  it('can be switched off', async () => {
    const api = await live();
    const s = JSON.parse(JSON.stringify(app.settings));
    s.swr.enabled = false;
    await app.saveSettings(s);
    api.setSwr(5);
    await app.send('CQ CQ CQ DE ON3VZ K');
    api.flush();
    api.emitSwr();
    api.emitSwr();
    await tick();
    expect(app.swrAlert).toBeNull();
  });
});

describe('decoder certainty meter', () => {
  it('stores the stats the decoder reports', async () => {
    const mock = await fresh();
    mock.setDecoder({ confidence: 0.82, snr_db: 9, accuracy: null });
    mock.flush();
    expect(app.decoder?.confidence).toBeCloseTo(0.82);
    expect(app.decoder?.accuracy).toBeNull();
  });
});

describe('live decoding', () => {
  beforeEach(async () => {
    await fresh();
    app.messages = [];
  });

  it('pieces of decoded text continue one bubble and fill {CALL}', () => {
    app.decoded('rx', 'CQ CQ DE ', [1, 1, 1, 1, 1, 1, 1, 1, 1], [], 1000);
    app.decoded('rx', 'F4XYZ K', [1, 1, 0.6, 1, 1, 1, 1], [], 3000);
    expect(app.messages).toHaveLength(1);
    expect(app.messages[0].text).toBe('CQ CQ DE F4XYZ K');
    expect(app.messages[0].conf?.[11]).toBeCloseTo(0.6);
    expect(app.messages[0].call).toBe('F4XYZ');
    expect(app.vars.CALL).toBe('F4XYZ');
  });

  it('a long pause starts a new bubble without a leading space', () => {
    app.decoded('rx', 'TEST', [1, 1, 1, 1], [], 1000);
    app.decoded('rx', ' AGN', [1, 1, 1, 1], [], 1000 + 9000);
    expect(app.messages.map((m) => m.text)).toEqual(['TEST', 'AGN']);
    expect(app.messages[1].conf).toHaveLength(3);
  });

  it('paddle text from the audio gets its own bubble', () => {
    app.decoded('rx', 'QRZ?', [1, 1, 1, 1], [], 1000);
    app.decoded('paddle', 'ON3VZ', [1, 1, 1, 1, 1], [], 2000);
    expect(app.messages.map((m) => m.from)).toEqual(['rx', 'paddle']);
  });

  it('the tentative tail is kept until final text replaces it', () => {
    app.decoded('rx', '', [], [{ best: 'K', confidence: 0.5, alts: [['R', 0.3]] }], 1000);
    expect(app.tail?.chars[0].best).toBe('K');
    app.decoded('rx', 'K', [0.95], [], 1500);
    expect(app.tail).toBeNull();
  });

  it('events from the core reach the conversation and the waterfall', async () => {
    const mock = await fresh();
    app.messages = [];
    mock.decode('rx', 'DE DL1ABC');
    mock.waterfallRow([0, 10, 200, 10]);
    mock.flush();
    expect(app.messages[0].text).toBe('DE DL1ABC');
    expect(app.waterfallLive).toBe(true);
    expect(app.waterfallRows).toHaveLength(1);
  });

  it('clicking a pitch retunes and remembers it, within limits', async () => {
    await app.setTone(615.4);
    expect(app.audio?.tone_hz).toBe(615);
    expect(app.settings?.audio.tone_hz).toBe(615);
    await app.setTone(5000);
    expect(app.audio?.tone_hz).toBe(1500);
  });

  it('recording starts and stops', async () => {
    await app.toggleRecording();
    expect(app.audio?.recording).toBe(true);
    expect(app.audio?.record_path).toMatch(/kHz\.wav$/);
    await app.toggleRecording();
    expect(app.audio?.recording).toBe(false);
  });
});

describe('conversations in tabs', () => {
  beforeEach(async () => {
    vi.useFakeTimers();
    await fresh();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it('starts with one live conversation that holds the demo text', () => {
    expect(app.sessions.length).toBe(1);
    expect(app.viewingLive).toBe(true);
    expect(app.messages.length).toBeGreaterThan(0);
  });

  it('a big frequency jump starts a new conversation and keeps the old one', () => {
    app.status.frequency_hz = 7_035_000;
    app.sessions[0].freq = 7_035_000;
    const oldId = app.cur.id;
    const before = app.messages.length;
    (app as unknown as { frequencyMoved(hz: number): void }).frequencyMoved(7_036_200);
    expect(app.sessions.length).toBe(1);
    vi.advanceTimersByTime(SETTLE_MS + 10);
    expect(app.sessions.length).toBe(2);
    expect(app.cur.id).not.toBe(oldId);
    expect(app.messages.length).toBe(0);
    app.show(oldId);
    expect(app.messages.length).toBe(before);
  });

  it('a small retune inside a signal does not start a new conversation', () => {
    app.sessions[0].freq = 7_035_000;
    (app as unknown as { frequencyMoved(hz: number): void }).frequencyMoved(7_035_200);
    expect(app.sessions.length).toBe(1);
  });

  it('turning the knob does not leave empty tabs behind', () => {
    app.sessions[0].freq = 7_035_000;
    const f = (hz: number) => (app as unknown as { frequencyMoved(hz: number): void }).frequencyMoved(hz);
    f(7_040_000);
    f(7_045_000);
    f(7_050_000);
    vi.advanceTimersByTime(SETTLE_MS + 10);
    expect(app.sessions.length).toBe(2);
    expect(app.cur.freq).toBe(7_050_000);
  });

  it('turning the knob over the band opens one tab at the end, not one per step', () => {
    app.sessions[0].freq = 7_035_000;
    for (const hz of [7_040_000, 7_050_000, 7_060_000, 14_020_000]) {
      app.status.frequency_hz = hz;
      (app as unknown as { frequencyMoved(hz: number): void }).frequencyMoved(hz);
      app.cur.messages.push({ id: 9000 + hz, from: 'rx', text: 'NOISE', time: '10:00' });
      vi.advanceTimersByTime(1000);
    }
    expect(app.sessions.length).toBe(1);
    vi.advanceTimersByTime(SETTLE_MS);
    expect(app.sessions.length).toBe(2);
    expect(app.cur.freq).toBe(14_020_000);
  });

  it('the wait before a new tab is a setting, 5 seconds by default', () => {
    expect(app.settings?.new_tab_seconds).toBe(5);
    app.settings!.new_tab_seconds = 2;
    app.sessions[0].freq = 7_035_000;
    (app as unknown as { frequencyMoved(hz: number): void }).frequencyMoved(7_100_000);
    vi.advanceTimersByTime(1900);
    expect(app.sessions.length).toBe(1);
    vi.advanceTimersByTime(300);
    expect(app.sessions.length).toBe(2);
  });

  it('coming back to the same frequency cancels the new tab', () => {
    app.sessions[0].freq = 7_035_000;
    const f = (hz: number) => (app as unknown as { frequencyMoved(hz: number): void }).frequencyMoved(hz);
    f(7_100_000);
    f(7_035_100);
    vi.advanceTimersByTime(SETTLE_MS * 2);
    expect(app.sessions.length).toBe(1);
  });

  it('a new callsign starts a new conversation and the bubble moves with it', () => {
    app.decoded('rx', 'CQ CQ DE F4XYZ F4XYZ K', [], [], Date.now() + 60_000);
    expect(app.sessions.length).toBe(2);
    expect(app.cur.qso.call).toBe('F4XYZ');
    expect(app.cur.messages.length).toBe(1);
    expect(app.sessions[0].qso.call).toBe('DL1ABC');
    expect(app.sessions[0].messages.some((m) => m.text.includes('F4XYZ'))).toBe(false);
  });

  it('the two conversations keep their own QSO details', () => {
    app.ingest('rx', 'DE F4XYZ NAME JEAN QTH PARIS K');
    expect(app.sessions.length).toBe(2);
    app.show(app.sessions[0].id);
    expect(app.qso.call).toBe('DL1ABC');
    expect(app.qso.name).toBe('HANS');
    app.qso.name = 'HANS-PETER';
    app.show(app.cur.id);
    expect(app.qso.call).toBe('F4XYZ');
    expect(app.qso.name).toBe('JEAN');
    expect(app.sessions[0].qso.name).toBe('HANS-PETER');
  });

  it('New QSO clears the screen but keeps the old conversation in its tab', () => {
    const before = app.messages.length;
    app.newConversation();
    expect(app.messages.length).toBe(0);
    expect(app.qso.call).toBe('');
    expect(app.sessions[0].messages.length).toBe(before);
  });

  it('New QSO on an empty conversation does not add a tab', () => {
    app.newConversation();
    app.newConversation();
    expect(app.sessions.length).toBe(2);
  });

  it('decoded text keeps going to the live conversation while you look back', () => {
    const oldId = app.cur.id;
    app.newConversation();
    app.show(oldId);
    app.decoded('rx', 'CQ TEST', [], []);
    expect(app.cur.messages.length).toBe(1);
    expect(app.viewingLive).toBe(false);
  });

  it('sending jumps to the live conversation', async () => {
    const oldId = app.cur.id;
    app.newConversation();
    app.show(oldId);
    await app.requestPolicy('practice');
    await app.send('CQ');
    expect(app.viewingLive).toBe(true);
  });

  it('closes an earlier conversation but never the live one', () => {
    const oldId = app.cur.id;
    app.newConversation();
    app.closeConversation(app.cur.id);
    expect(app.sessions.length).toBe(2);
    app.closeConversation(oldId);
    expect(app.sessions.length).toBe(1);
  });

  it('keeps at most 40 conversations', () => {
    for (let i = 0; i < 60; i++) {
      app.ingest('rx', `DE F${i % 9}AB${String.fromCharCode(65 + (i % 26))} K`);
      app.cur.messages.push({ id: 5000 + i, from: 'tx', text: 'X', time: '10:00' });
      app.newConversation();
    }
    expect(app.sessions.length).toBeLessThanOrEqual(40);
  });
});

describe('signal threshold and indicator', () => {
  it('the squelch is remembered and limited to 1 to 8', async () => {
    await fresh();
    await app.setSquelch(3.25);
    expect(app.settings?.audio.squelch).toBe(3.3);
    await app.setSquelch(99);
    expect(app.settings?.audio.squelch).toBe(8);
    await app.setSquelch(-5);
    expect(app.settings?.audio.squelch).toBe(1);
  });

  it('the indicator follows the decoder report', async () => {
    await fresh();
    app.decoder = { confidence: 0.9, snr_db: 12, accuracy: null };
    expect(app.signalPresent).toBe(true);
    app.decoder = { confidence: 0.9, snr_db: null, accuracy: null };
    expect(app.signalPresent).toBe(false);
  });
});

describe('conversations survive a restart', () => {
  it('saves what was heard and restores it as earlier conversations', async () => {
    const store = new Map<string, string>();
    const fake = { getItem: (k: string) => store.get(k) ?? null, setItem: (k: string, v: string) => void store.set(k, v), removeItem: (k: string) => void store.delete(k) };
    vi.stubGlobal('localStorage', fake);
    vi.useFakeTimers();
    try {
      const api = new MockApi({ station: { calls: ['ON3VZ'], name: 'K', qth: 'H', locator: 'JO20EE' } });
      Object.defineProperty(api, 'demo', { value: false });
      await app.init(api);
      app.decoded('rx', 'CQ CQ DE OK5O OK5O K', [], []);
      vi.advanceTimersByTime(2000);
      expect(store.get('cwld.sessions.v1')).toContain('OK5O');
      const api2 = new MockApi({ station: { calls: ['ON3VZ'], name: 'K', qth: 'H', locator: 'JO20EE' } });
      Object.defineProperty(api2, 'demo', { value: false });
      await app.init(api2);
      expect(app.sessions.length).toBe(2);
      expect(app.cur.messages.length).toBe(0);
      expect(app.sessions[0].qso.call).toBe('OK5O');
    } finally {
      vi.useRealTimers();
      vi.unstubAllGlobals();
    }
  });
});

describe('updates', () => {
  it('stays quiet in demo mode and learns the version', async () => {
    await fresh();
    await Promise.resolve();
    expect(app.updateState).toBe('idle');
    expect(app.version).toBe('0.0.0-demo');
  });

  it('says so when this is the newest version', async () => {
    await fresh();
    await app.checkUpdate();
    expect(app.updateState).toBe('none');
  });

  it('offers a newer version with its notes and installs it', async () => {
    const api = await fresh();
    api.offered = { version: '9.9.9', notes: 'Better decoder' };
    await app.checkUpdate();
    expect(app.updateState).toBe('available');
    expect(app.updateInfo?.notes).toBe('Better decoder');
    await app.installUpdate();
    expect(api.installed).toBe(true);
    expect(app.updateProgress).toBe(1);
  });

  it('shows an error on a manual check but not on the quiet start check', async () => {
    const api = await fresh();
    api.checkUpdate = async () => {
      throw new Error('offline');
    };
    await app.checkUpdate(true);
    expect(app.updateState).toBe('idle');
    await app.checkUpdate();
    expect(app.updateState).toBe('error');
  });
});

describe('band scope', () => {
  it('shows sweeps from the radio and a click tunes the radio there', async () => {
    const api = await fresh();
    expect(app.scopeLive).toBe(false);
    (api as unknown as { pending: unknown[] }).pending.push(api.scopeSweep());
    api.flush();
    expect(app.scopeLive).toBe(true);
    expect(app.scopeHi - app.scopeLo).toBe(50_000);
    await app.tuneTo(7_023_604);
    expect(app.status.frequency_hz).toBe(7_023_600);
  });

  it('hiding the band scope or the audio waterfall is remembered', async () => {
    await fresh();
    await app.setView('band_scope', false);
    await app.setView('audio_waterfall', false);
    expect(app.settings?.band_scope).toBe(false);
    expect(app.settings?.audio_waterfall).toBe(false);
    await app.setView('band_scope', true);
    expect(app.settings?.band_scope).toBe(true);
  });
});

describe('inserting codes from the library', () => {
  it('adds with a space before and after, in capitals', () => {
    app.draft = '';
    app.insertText('qth?');
    expect(app.draft).toBe('QTH? ');
    app.insertText('<AR>');
    expect(app.draft).toBe('QTH? <AR> ');
    app.draft = 'DL1ABC';
    app.insertText('TNX');
    expect(app.draft).toBe('DL1ABC TNX ');
  });
});

describe('report, pile-up keys and contest number', () => {
  beforeEach(async () => {
    await fresh();
  });

  it('the report is 5NN by default and 599 when the setting says so', async () => {
    expect(app.cur.qso.rst).toBe('5NN');
    await app.setDefaultRst('599');
    app.newConversation(7_040_000);
    app.cur.messages.push({ id: 9999, from: 'rx', text: 'CQ', time: '10:00' });
    app.newConversation(7_041_000);
    expect(app.cur.qso.rst).toBe('599');
  });

  it('the pile-up keys send a fixed text with the report from the QSO panel', async () => {
    await app.requestPolicy('practice');
    app.cur.qso.rst = '5NN';
    const spy = vi.spyOn(app, 'sendChip');
    await app.pressPileup('F3');
    expect(spy).toHaveBeenCalledWith('R {RST} TU');
    spy.mockClear();
    await app.requestPolicy('listen_only');
    await app.pressPileup('F1');
    expect(spy).not.toHaveBeenCalled();
  });

  it('{NR} has a value in contest mode only, and a stored number asks to continue or restart', async () => {
    expect(app.vars.NR).toBe('');
    await app.setContestNr(5);
    await app.requestContest(true);
    expect(app.contestAsk).toBe(true);
    await app.answerContest(false);
    expect(app.settings?.contest_mode).toBe(true);
    expect(app.vars.NR).toBe('005');
    await app.requestContest(false);
    expect(app.vars.NR).toBe('');
    await app.requestContest(true);
    await app.answerContest(true);
    expect(app.vars.NR).toBe('001');
  });

  it('contest mode on with number 1 needs no question', async () => {
    await app.requestContest(true);
    expect(app.contestAsk).toBe(false);
    expect(app.settings?.contest_mode).toBe(true);
  });
});

describe('tabs: jump to the frequency, close idle ones, close all', () => {
  beforeEach(async () => {
    vi.useFakeTimers();
    await fresh();
    app.sessions[0].freq = 7_030_000;
    app.sessions[0].messages = [];
    app.status.frequency_hz = 7_030_000;
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  function twoTabs() {
    const first = app.cur.id;
    app.cur.messages.push({ id: 9001, from: 'rx', text: 'CQ DE DL1ABC', time: '10:00' });
    app.status.frequency_hz = 14_030_000;
    app.newConversation(14_030_000);
    app.cur.messages.push({ id: 9002, from: 'rx', text: 'CQ DE F5XYZ', time: '10:01' });
    return first;
  }

  it('clicking an earlier tab makes it live again and tunes the radio to its frequency', async () => {
    const first = twoTabs();
    const api = (app as unknown as { api: MockApi }).api;
    const spy = vi.spyOn(api, 'setFrequency');
    await app.openTab(first);
    expect(app.cur.id).toBe(first);
    expect(spy).toHaveBeenCalledWith(7_030_000);
    // The jump itself does not open another tab.
    vi.advanceTimersByTime(SETTLE_MS + 10);
    expect(app.cur.id).toBe(first);
    expect(app.sessions.length).toBe(2);
  });

  it('does not tune while transmitting, it only shows the tab', async () => {
    const first = twoTabs();
    const live = app.cur.id;
    app.status.transmitting = true;
    await app.openTab(first);
    expect(app.shown.id).toBe(first);
    expect(app.cur.id).toBe(live);
  });

  it('without a radio the tab is only shown', async () => {
    const first = twoTabs();
    const live = app.cur.id;
    app.status.connected = false;
    await app.openTab(first);
    expect(app.shown.id).toBe(first);
    expect(app.cur.id).toBe(live);
  });

  it('idle tabs close after the set time, but not the live one, the shown one or one with sent text', () => {
    const first = twoTabs();
    const live = app.cur.id;
    // A third tab with sent text that is not logged.
    app.newConversation(21_030_000);
    app.cur.messages.push({ id: 9003, from: 'tx', text: 'CQ DE ON3VZ', time: '10:02' });
    const third = app.cur.id;
    app.newConversation(28_030_000);
    app.cur.messages.push({ id: 9004, from: 'rx', text: 'CQ', time: '10:03' });
    const fourth = app.cur.id;
    const now = Date.now() + 16 * 60_000;
    app.sweepTabs(now);
    const ids = app.sessions.map((s) => s.id);
    expect(ids).not.toContain(first);
    expect(ids).not.toContain(live);
    expect(ids).toContain(third);
    expect(ids).toContain(fourth);
  });

  it('0 minutes switches the closing off', async () => {
    const first = twoTabs();
    app.settings!.tabs_auto_close_minutes = 0;
    app.sweepTabs(Date.now() + 600 * 60_000);
    expect(app.sessions.map((s) => s.id)).toContain(first);
  });

  it('close all asks first when sent text is not logged, and keeps one empty tab', () => {
    twoTabs();
    app.cur.messages.push({ id: 9005, from: 'tx', text: 'CQ', time: '10:05' });
    app.requestCloseAll();
    expect(app.closeAllAsk).toBe(1);
    expect(app.sessions.length).toBe(2);
    app.closeAllNow();
    expect(app.closeAllAsk).toBe(0);
    expect(app.sessions.length).toBe(1);
    expect(app.cur.messages.length).toBe(0);
    expect(app.viewingLive).toBe(true);
  });

  it('close all needs no question when nothing was sent', () => {
    twoTabs();
    app.requestCloseAll();
    expect(app.sessions.length).toBe(1);
    expect(app.closeAllAsk).toBe(0);
  });
});

describe('decoder tuning and presets', () => {
  beforeEach(async () => {
    await fresh();
  });

  it('gain, filter and speed range are limited and remembered', async () => {
    await app.setTuning({ gain_db: 55, wide_filter: true, min_wpm: 3, max_wpm: 100 });
    const a = app.settings!.audio;
    expect(a.gain_db).toBe(20);
    expect(a.wide_filter).toBe(true);
    expect(a.min_wpm).toBe(5);
    expect(a.max_wpm).toBe(60);
    await app.setTuning({ lock_wpm: 3 });
    expect(app.settings!.audio.lock_wpm).toBe(0);
    await app.setTuning({ lock_wpm: 18 });
    expect(app.settings!.audio.lock_wpm).toBe(18);
  });

  it('the slowest speed keeps at least 5 WPM below the fastest', async () => {
    await app.setTuning({ min_wpm: 40, max_wpm: 20 });
    expect(app.settings!.audio.max_wpm - app.settings!.audio.min_wpm).toBeGreaterThanOrEqual(5);
  });

  it('presets set the squelch and the matching one is marked', async () => {
    await app.setPreset('strong');
    expect(app.settings!.audio.squelch).toBe(5.5);
    expect(app.preset).toBe('strong');
    await app.setSquelch(4.1);
    expect(app.preset).toBeNull();
  });

  it('strongest and AFC reach the core', async () => {
    const api = (app as unknown as { api: MockApi }).api;
    await app.goStrongest();
    expect(api.strongestCalls).toBe(1);
    await app.setAfc(false);
    expect(app.settings!.audio.afc).toBe(false);
  });

  it('the peak and the filter width come from the core events', () => {
    (app as unknown as { onEvents(e: unknown[]): void }).onEvents([
      { type: 'audio_level', dbfs: -20, peak: 0.42 },
      { type: 'bandwidth', hz: 25 },
    ]);
    expect(app.audioPeak).toBeCloseTo(0.42);
    expect(app.bandwidthHz).toBe(25);
  });

  it('a decoder hold is shown and cleared', () => {
    const on = (e: unknown[]) => (app as unknown as { onEvents(e: unknown[]): void }).onEvents(e);
    on([{ type: 'decode_hold', hold: 'mode', mode: 'USB' }]);
    expect(app.decodeHold).toEqual({ hold: 'mode', mode: 'USB' });
    on([{ type: 'decode_hold', hold: null, mode: 'CW' }]);
    expect(app.decodeHold).toBeNull();
  });

  it('own words are added, replaced and removed', async () => {
    await app.addUserWord('tnx2u', 'Thanks to you');
    expect(app.settings!.user_words).toEqual([{ code: 'TNX2U', meaning: 'Thanks to you' }]);
    await app.addUserWord('TNX2U', 'Many thanks');
    expect(app.settings!.user_words.length).toBe(1);
    expect(app.settings!.user_words[0].meaning).toBe('Many thanks');
    await app.removeUserWord('TNX2U');
    expect(app.settings!.user_words.length).toBe(0);
  });
});

describe('contest profiles in the app', () => {
  beforeEach(async () => {
    await fresh();
  });

  it('own variables are available in templates, with the defaults for Belgium', () => {
    expect(app.vars.SEC).toBe('WLD');
    expect(app.vars.ZONE).toBe('14');
    expect(app.vars.ITU).toBe('27');
    expect(app.vars.PWR).toBe('20');
  });

  it('choosing a profile gives its call chips and its exchange, and shows its note', async () => {
    await app.requestContest(true);
    await app.selectContestProfile('uba-dx-cw');
    const chips = app.contestChipList;
    expect(chips.map((c) => c.label)).toContain('TEST UBA');
    expect(chips.find((c) => c.id === 'exchange')?.template).toBe('{CALL} {RST} {NR} {SEC}');
    expect(app.contestNote).not.toBeNull();
    await app.setContestSent(1);
    expect(app.contestChipList.find((c) => c.id === 'exchange')?.template).toBe('{CALL} {RST} {NR}');
    await app.selectContestProfile('cqwpx-cw');
    expect(app.settings?.contest.sent_index).toBe(0);
  });

  it('without a profile the plain exchange chip stays', () => {
    expect(app.contestProfile).toBeNull();
    expect(app.contestChipList.map((c) => c.id)).toEqual(['exchange']);
  });

  it('the serial number is shown only when the exchange uses it, and as cut numbers when asked', async () => {
    await app.requestContest(true);
    await app.setContestNr(19);
    await app.selectContestProfile('cqww-cw');
    expect(app.serialUsed).toBe(false);
    await app.selectContestProfile('cqwpx-cw');
    expect(app.serialUsed).toBe(true);
    expect(app.vars.NR).toBe('019');
    const id = await app.copyContestProfile('cqwpx-cw', 'Cut');
    const p = app.settings!.contest.profiles.find((x) => x.id === id)!;
    await app.saveContestProfile({ ...p, cut_numbers: true });
    await app.selectContestProfile(id!);
    expect(app.vars.NR).toBe('T1N');
    expect(app.serialText).toBe('T1N');
  });

  it('own profiles can be made, changed, copied and deleted; built-in ones cannot', async () => {
    const id = (await app.newContestProfile('Club'))!;
    expect(app.settings!.contest.profiles.map((p) => p.name)).toEqual(['Club']);
    const p = app.settings!.contest.profiles[0];
    await app.saveContestProfile({ ...p, name: 'Club 2' });
    expect(app.settings!.contest.profiles[0].name).toBe('Club 2');
    await app.saveContestProfile({ ...p, id: 'cqww-cw', name: 'Hacked' });
    expect(app.settings!.contest.profiles).toHaveLength(1);
    await app.selectContestProfile(id);
    await app.deleteContestProfile('cqww-cw');
    expect(app.settings!.contest.profiles).toHaveLength(1);
    await app.deleteContestProfile(id);
    expect(app.settings!.contest.profiles).toHaveLength(0);
    expect(app.settings!.contest.active).toBe('');
  });

  it('own variables can be changed and a bad import gives an error and changes nothing', async () => {
    await app.saveContestVars([{ name: 'AGE', value: '55' }]);
    expect(app.vars.AGE).toBe('55');
    expect(app.vars.SEC).toBeUndefined();
    const r = await app.importContest('not json', false);
    expect(r).toBeNull();
    expect(app.error).not.toBeNull();
    expect(app.settings!.contest.vars).toEqual([{ name: 'AGE', value: '55' }]);
  });

  it('a backup can be restored by importing it', async () => {
    const id = (await app.newContestProfile('Backup me'))!;
    await app.saveContestVars([{ name: 'AGE', value: '55' }]);
    const text = (await import('./contest')).exportText(app.settings!.contest);
    await app.deleteContestProfile(id);
    await app.saveContestVars([]);
    const r = await app.importContest(text, false);
    expect(r).toEqual({ added: 1, skipped: 0 });
    expect(app.settings!.contest.profiles.map((p) => p.name)).toEqual(['Backup me']);
    expect(app.vars.AGE).toBe('55');
  });
});

describe('logbook and contest entry', () => {
  beforeEach(async () => {
    await fresh();
    await app.requestContest(true);
    await app.selectContestProfile('uba-dx-cw');
    app.clearEntry();
  });

  it('logging writes the QSO, raises the number and clears the line', async () => {
    app.selectCall('pa3abc');
    app.entry.nr = '012';
    expect(app.entryComplete).toBe(true);
    expect(await app.logQso()).toBe(true);
    expect(app.recent).toHaveLength(1);
    expect(app.recent[0]).toMatchObject({ call: 'PA3ABC', nr_sent: 1, nr_rcvd: '012', contest: 'uba-dx-cw', test: false, extra_sent: 'WLD' });
    expect(app.settings?.contest_nr).toBe(2);
    expect(app.qso.call).toBe('');
    expect(app.qsoCount).toBe(1);
  });

  it('refuses to log without a call', async () => {
    expect(await app.logQso()).toBe(false);
    expect(app.recent).toHaveLength(0);
    expect(app.settings?.contest_nr).toBe(1);
  });

  it('the same call on the same band again is a dupe, a Belgian station needs its section', async () => {
    app.selectCall('ON4XYZ');
    expect(app.entryComplete).toBe(false);
    app.entry.nr = '5';
    app.entry.extra = 'ANR';
    expect(app.entryComplete).toBe(true);
    await app.logQso();
    app.selectCall('ON4XYZ');
    await app.refreshDupe();
    expect(app.dupeInfo?.dupe?.call).toBe('ON4XYZ');
    app.selectCall('PA3ABC');
    await app.refreshDupe();
    expect(app.dupeInfo?.dupe).toBeNull();
  });

  it('a new contest starts at 001 and earlier QSOs no longer count as dupes', async () => {
    app.selectCall('PA3ABC');
    app.entry.nr = '1';
    await app.logQso();
    await app.requestContest(false);
    await app.requestContest(true);
    expect(app.contestAsk).toBe(true);
    await app.answerContest(true);
    expect(app.settings?.contest_nr).toBe(1);
    expect(app.contestSession).toBe(2);
    app.selectCall('PA3ABC');
    await app.refreshDupe();
    expect(app.dupeInfo?.dupe).toBeNull();
    expect(app.qsoCount).toBe(0);
  });

  it('test mode logs apart with its own number and leaves the real number alone', async () => {
    await app.setTestMode(true);
    expect(app.serialText).toBe('001');
    app.selectCall('PA3ABC');
    app.entry.nr = '1';
    await app.logQso();
    expect(app.testNr).toBe(2);
    expect(app.serialText).toBe('002');
    expect(app.settings?.contest_nr).toBe(1);
    expect(app.recent[0].test).toBe(true);
    await app.clearTestLog();
    expect(app.qsoCount).toBe(0);
    await app.setTestMode(false);
    expect(app.serialText).toBe('001');
  });

  it('changing the call after the exchange was sent warns about the unlogged number', async () => {
    await app.requestPolicy('practice');
    app.selectCall('PA3ABC');
    await app.pressContestKey('F2');
    expect(app.sentPending).toEqual({ call: 'PA3ABC', nr: '001' });
    app.selectCall('DL1XYZ');
    expect(app.warning).toContain('001');
    expect(app.warning).toContain('PA3ABC');
    expect(app.sentPending).toBeNull();
  });

  it('sending the exchange does not raise the number, logging does', async () => {
    await app.requestPolicy('practice');
    app.selectCall('PA3ABC');
    await app.pressContestKey('F2');
    await app.pressContestKey('F2');
    expect(app.settings?.contest_nr).toBe(1);
    app.entry.nr = '9';
    await app.logQso();
    expect(app.settings?.contest_nr).toBe(2);
  });

  it('Enter in Run: CQ, exchange, AGN, then TU with log once the reply is complete', async () => {
    await app.requestPolicy('practice');
    await app.pressEnter();
    expect(app.messages[app.messages.length - 1]?.text).toContain('TEST UBA');
    app.selectCall('PA3ABC');
    await app.pressEnter();
    expect(app.exchangeSent).toBe(true);
    expect(app.messages[app.messages.length - 1]?.text).toContain('PA3ABC 5NN 001 WLD');
    await app.pressEnter();
    expect(app.messages[app.messages.length - 1]?.text).toBe('AGN');
    app.entry.nr = '33';
    await app.pressEnter();
    expect(app.recent[0]?.call).toBe('PA3ABC');
    expect(app.settings?.contest_nr).toBe(2);
  });

  it('proposes report and number from the reply after our exchange, and Tab takes them over', async () => {
    await app.requestPolicy('practice');
    app.selectCall('PA3ABC');
    await app.pressContestKey('F2');
    app.ingest('rx', 'PA3ABC DE ON3VZ 5NN 023');
    expect(app.suggestion).toMatchObject({ rst: '5NN', nr: '023' });
    app.acceptSuggestion();
    expect(app.entry.nr).toBe('023');
  });

  it('a word clicked in the decoded text goes where the operator says', () => {
    app.assignWord('anr', 'extra');
    app.assignWord('T2N', 'nr');
    app.assignWord('hans', 'name');
    expect(app.entry.extra).toBe('ANR');
    expect(app.entry.nr).toBe('029');
    expect(app.qso.name).toBe('HANS');
  });

  it('outside contest mode a normal QSO is logged without a number', async () => {
    await app.requestContest(false);
    app.selectCall('F4XYZ');
    app.entry.rst = '579';
    await app.logQso();
    expect(app.recent).toHaveLength(1);
    expect(app.recent[0]).toMatchObject({ call: 'F4XYZ', contest: '', nr_sent: 0, rst_rcvd: '579' });
    expect(app.settings?.contest_nr).toBe(1);
  });
});

describe('station profiles and activations (C-3)', () => {
  beforeEach(async () => {
    await fresh();
  });

  it('switches profile and the on-air call gets the suffix', async () => {
    const portable = app.settings!.profiles.find((p) => p.suffix)!;
    await app.setProfile(portable.id);
    expect(app.settings!.profile).toBe(portable.id);
    expect(app.myCall).toBe(`ON3VZ${portable.suffix}`.toUpperCase());
  });

  it('refuses a portable activation without a grid and a bad reference', async () => {
    const portable = app.settings!.profiles.find((p) => p.suffix)!;
    await app.setProfile(portable.id);
    expect(await app.startActivation({ grid: '' })).toBe('activation_grid_missing');
    expect(await app.startActivation({ grid: 'JO20QE', wwff: 'ONFF599' })).toBe('activation_wwff');
    expect(await app.startActivation({ grid: 'ZZ99', wwff: '' })).toBe('activation_grid');
    expect(app.activation).toBeNull();
  });

  it('starts an activation, locks the profile and ends it again', async () => {
    const portable = app.settings!.profiles.find((p) => p.suffix)!;
    await app.setProfile(portable.id);
    expect(await app.startActivation({ grid: 'jo20qe', wwff: 'onff-0599', power: '5' })).toBeNull();
    expect(app.activation?.wwff).toBe('ONFF-0599');
    expect(app.activation?.grid).toBe('JO20QE');
    const home = app.settings!.profiles.find((p) => !p.suffix)!;
    await app.setProfile(home.id);
    expect(app.settings!.profile).toBe(portable.id);
    await app.endActivation();
    expect(app.activation).toBeNull();
    await app.setProfile(home.id);
    expect(app.settings!.profile).toBe(home.id);
  });

  it('stores the scope sizes', async () => {
    await app.setScopeSize('scope_size', 'large');
    await app.setScopeSize('scope_size_contest', 'small');
    expect(app.settings!.scope_size).toBe('large');
    expect(app.settings!.scope_size_contest).toBe('small');
  });
});

describe('practice simulator (O)', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('needs practice mode', async () => {
    await fresh();
    await app.requestPolicy('listen_only');
    await app.startExercise();
    expect(app.practiceRun).toBeNull();
    expect(app.error).toBeTruthy();
  });

  it('runs a QSO against a simulated station, logs it in the test log and scores it', async () => {
    vi.useFakeTimers();
    const api = await fresh();
    await app.requestPolicy('practice');
    await app.setPracticeCfg({ exercise: 'they_cq', decoder_help: true });
    await app.startExercise();
    expect(app.practiceRun?.exercise).toBe('they_cq');
    expect(app.testMode).toBe(true);
    expect(api.practiceLog).toHaveLength(1);
    const call = api.practiceLog[0].text.split(' DE ')[1].split(' ')[0];
    await vi.advanceTimersByTimeAsync(20);
    api.practiceEndAll();
    api.flush();
    await app.send(`${call} DE ON3VZ ON3VZ K`);
    await vi.advanceTimersByTimeAsync(30_000);
    expect(api.practiceLog).toHaveLength(2);
    const info = api.practiceLog[1].text;
    expect(info).toContain(`ON3VZ DE ${call}`);
    api.practiceEndAll();
    api.flush();
    await vi.advanceTimersByTimeAsync(20);
    const name = info.split(' NAME ')[1].split(' ')[0];
    app.qso = { ...app.qso, call, name, qth: '' };
    app.entry = { ...app.entry, rst: info.split(' RST ')[1].split(' ')[0] };
    expect(await app.logQso()).toBe(true);
    await app.stopExercise();
    expect(app.practiceRun).toBeNull();
    expect(api.practiceRunning).toBe(false);
    expect(app.testMode).toBe(false);
    expect(app.practiceScore?.logged).toBe(1);
    expect(app.practiceScore?.correct).toBe(1);
    expect(api.transmitCommands).toBe(0);
  });

  it('leaving practice mode ends the exercise', async () => {
    await fresh();
    await app.requestPolicy('practice');
    await app.startExercise();
    expect(app.practiceRun).not.toBeNull();
    await app.requestPolicy('listen_only');
    expect(app.practiceRun).toBeNull();
  });

  it('hides decoded text without decoder help', async () => {
    await fresh();
    await app.requestPolicy('practice');
    await app.setPracticeCfg({ decoder_help: false });
    await app.startExercise();
    expect(app.hideRx).toBe(true);
    await app.stopExercise();
    expect(app.hideRx).toBe(false);
  });
});

describe('contest practice (O)', () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it('F1 calls CQ and simulated callers answer', async () => {
    vi.useFakeTimers();
    const api = await fresh();
    await app.requestPolicy('practice');
    await app.selectContestProfile('cqwpx-cw');
    await app.setPracticeCfg({ exercise: 'contest', pileup: 3 });
    await app.startExercise();
    expect(app.settings!.contest_mode).toBe(true);
    for (let i = 0; i < 5 && api.practiceLog.length === 0; i++) {
      await app.pressContestKey('F1');
      expect(app.practiceBusy).toBe(true);
      await vi.advanceTimersByTimeAsync(15_000);
    }
    expect(api.practiceLog.length).toBeGreaterThan(0);
    await app.stopExercise();
    expect(app.settings!.contest_mode).toBe(false);
  });
});

describe('guided first start', () => {
  it('stays closed in the browser demo', async () => {
    await fresh();
    expect(app.wizardOpen).toBe(false);
  });

  it('opens on a real first run and not once it has been done', async () => {
    const first = new MockApi();
    Object.defineProperty(first, 'demo', { value: false });
    await app.init(first);
    expect(app.wizardOpen).toBe(true);
    const done = new MockApi({ first_run_done: true });
    Object.defineProperty(done, 'demo', { value: false });
    await app.init(done);
    expect(app.wizardOpen).toBe(false);
  });
});

describe('UTC times', () => {
  it('shows conversation times in UTC, not local time', async () => {
    const { utcHHMM } = await import('./app.svelte');
    expect(utcHHMM(Date.UTC(2026, 10, 9, 23, 59, 30))).toBe('23:59');
    expect(utcHHMM(Date.UTC(2026, 0, 1, 0, 5, 0))).toBe('00:05');
  });
});

describe('stop and emergency stop', () => {
  async function live() {
    const api = await fresh();
    await app.requestPolicy('live');
    await app.confirmLiveSwitch();
    return api;
  }

  it('one Esc stops and sends nothing more', async () => {
    const api = await live();
    await app.send('CQ CQ DE ON3VZ K');
    app.status.transmitting = true; // the radio reports it while it keys
    const before = api.transmitCommands;
    await app.stop();
    expect(api.transmitCommands).toBe(before);
    expect(app.notice).toMatch(/QRX/);
  });

  it('a second Esc within two seconds stops again and sends QRX QRX', async () => {
    const api = await live();
    await app.send('CQ CQ DE ON3VZ K');
    app.status.transmitting = true; // the radio reports it while it keys
    const before = api.transmitCommands;
    await app.stop();
    await app.stop();
    expect(api.transmitCommands).toBe(before + 1);
    expect(app.cur.messages[app.cur.messages.length - 1]?.text).toBe('QRX QRX');
  });

  it('a second Esc after two seconds sends nothing', async () => {
    vi.useFakeTimers();
    try {
      const api = await live();
      await app.send('CQ CQ DE ON3VZ K');
    app.status.transmitting = true; // the radio reports it while it keys
      const before = api.transmitCommands;
      await app.stop();
      vi.setSystemTime(Date.now() + 2500);
      await app.stop();
      expect(api.transmitCommands).toBe(before);
    } finally {
      vi.useRealTimers();
    }
  });

  it('Esc when nothing was being sent never sends QRX', async () => {
    const api = await live();
    const before = api.transmitCommands;
    await app.stop();
    await app.stop();
    expect(api.transmitCommands).toBe(before);
  });

  it('never transmits in listen mode', async () => {
    const api = await fresh();
    await app.stopAndQrx();
    await app.sendError();
    expect(api.transmitCommands).toBe(0);
  });

  it('the Stop + QRX button sends QRX QRX and keeps the draft', async () => {
    const api = await live();
    app.draft = 'TNX FER CALL';
    await app.stopAndQrx();
    expect(api.transmitCommands).toBe(1);
    expect(app.draft).toBe('TNX FER CALL');
  });

  it('the error button sends the error prosign', async () => {
    const api = await live();
    await app.send('DL1ABC DE ON3VZ');
    const before = api.transmitCommands;
    await app.sendError();
    expect(api.transmitCommands).toBe(before + 1);
    expect(app.cur.messages[app.cur.messages.length - 1]?.text).toBe('<HH>');
  });
});

describe('logging link', () => {
  it('settings default to off and a resend without a real QSO is refused', async () => {
    const api = new MockApi();
    expect(defaultSettings().link.adif_enabled).toBe(false);
    await expect(api.linkResendLast()).rejects.toBeTruthy();
  });
});

describe('paddle trainer', () => {
  const on = (e: unknown[]) => (app as unknown as { onEvents(e: unknown[]): void }).onEvents(e);
  /** Key runs at 20 WPM for a text, as the core would hand them out. */
  const runs = (text: string): [boolean, number][] => {
    const P: Record<string, string> = { K: '-.-', M: '--', R: '.-.', S: '...', U: '..-', A: '.-', P: '.--.', T: '-', L: '.-..', O: '---', W: '.--', I: '..', N: '-.', J: '.---', E: '.', F: '..-.', '0': '-----', Y: '-.--', V: '...-', G: '--.', '5': '.....', '/': '-..-.', Q: '--.-', '9': '----.', Z: '--..', H: '....', '3': '...--', '8': '---..', B: '-...', '?': '..--..', '4': '....-', '2': '..---', '7': '--...', C: '-.-.', '1': '.----', D: '-..', '6': '-....', X: '-..-' };
    const out: [boolean, number][] = [];
    text.split(' ').forEach((w, wi) => {
      if (wi) out.push([false, 420]);
      [...w].forEach((c, ci) => {
        if (ci) out.push([false, 180]);
        [...P[c]].forEach((e, ei) => {
          if (ei) out.push([false, 60]);
          out.push([true, e === '.' ? 60 : 180]);
        });
      });
    });
    return out;
  };

  it('needs practice mode, scores a keyed text and remembers the result', async () => {
    const api = await fresh();
    await app.paddleStart('chars');
    expect(app.paddleEx).toBeNull();
    await app.requestPolicy('practice');
    await app.paddleStart('chars');
    expect(api.keying).toBe(true);
    const target = app.paddleEx!.target;
    on([{ type: 'keying', runs: runs(target) }]);
    expect(app.paddleEx!.heard).toBe(true);
    on([{ type: 'decode', source: 'rx', text: target, conf: [], tentative: [], wpm: 20 }]);
    // The decoded text does not end up in the conversation.
    expect(app.messages.some((m) => m.text.includes(target.slice(0, 5)))).toBe(false);
    on([{ type: 'keying', runs: [[false, 2500]] }]);
    await vi.waitFor(() => expect(app.paddleEx!.result).not.toBeNull());
    const r = app.paddleEx!.result!;
    expect(r.accuracy).toBe(1);
    expect(r.timing.wpm).toBeCloseTo(20, 0);
    expect(r.score).toBeGreaterThan(90);
    await vi.waitFor(() => expect(app.settings!.practice.paddle.sessions.length).toBe(1));
    expect(app.settings!.practice.paddle.stats.reduce((a, s) => a + s.tries, 0)).toBe(target.replaceAll(' ', '').length);
    app.paddleNext();
    expect(app.paddleEx!.result).toBeNull();
    await app.requestPolicy('listen_only');
    expect(app.paddleEx).toBeNull();
    expect(api.keying).toBe(false);
  });

  it('shows when practice mode quieted the radio', () => {
    on([{ type: 'radio_quiet', quiet: true }]);
    expect(app.radioQuiet).toBe(true);
    on([{ type: 'radio_quiet', quiet: false }]);
    expect(app.radioQuiet).toBe(false);
  });
});

describe('QRZ.com lookup', () => {
  async function withLookup(enabled = true) {
    const api = new MockApi({ station: { calls: ['ON3VZ'], name: 'KRISTOF', qth: 'HOBOKEN', locator: 'JO20EE' }, lookup: { qrz_enabled: enabled, qrz_user: 'ON3VZ' } });
    api.lookupPassword = 'geheim';
    await app.init(api);
    return api;
  }

  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('settings default to off and carry no password', () => {
    const d = defaultSettings();
    expect(d.lookup).toEqual({ qrz_enabled: false, qrz_user: '' });
    expect(JSON.stringify(d)).not.toMatch(/password/i);
  });

  it('fills empty name and QTH from the answer after the call stops changing', async () => {
    const api = await withLookup();
    app.selectCall('ON4ZZ');
    app.qso = { ...app.qso, name: '', qth: '' };
    await vi.advanceTimersByTimeAsync(800);
    expect(api.lookupCalls).toContain('ON4ZZ');
    expect(app.qso.name).toBe('Jan');
    expect(app.qso.qth).toBe('Gent');
    expect(app.lookupNote?.text).toContain('Gent');
  });

  it('never replaces what the operator typed', async () => {
    await withLookup();
    app.selectCall('ON4ZZ');
    app.qso = { ...app.qso, name: 'JEF', qth: '' };
    await vi.advanceTimersByTimeAsync(800);
    expect(app.qso.name).toBe('JEF');
    expect(app.qso.qth).toBe('Gent');
  });

  it('asks nothing while switched off and waits while typing', async () => {
    const off = await withLookup(false);
    app.selectCall('ON4ZZ');
    await vi.advanceTimersByTimeAsync(800);
    expect(off.lookupCalls).toEqual([]);
    const on = await withLookup(true);
    app.selectCall('ON4');
    app.selectCall('ON4Z');
    app.selectCall('ON4ZZ');
    await vi.advanceTimersByTimeAsync(800);
    expect(on.lookupCalls).toEqual(['ON4ZZ']);
  });

  it('a failed lookup warns once and does not touch the fields', async () => {
    const api = await withLookup();
    api.lookupPassword = '';
    app.selectCall('ON4ZZ');
    app.qso = { ...app.qso, name: '', qth: '' };
    await vi.advanceTimersByTimeAsync(800);
    expect(app.qso.name).toBe('');
    expect(app.warning).toBeTruthy();
  });
});

describe('QRZ.com lookup when the call is typed (0.13.1)', () => {
  async function withLookup() {
    const api = new MockApi({ station: { calls: ['ON3VZ'], name: 'KRISTOF', qth: 'HOBOKEN', locator: 'JO20EE' }, lookup: { qrz_enabled: true, qrz_user: 'ON3VZ' } });
    api.lookupPassword = 'geheim';
    await app.init(api);
    return api;
  }
  /** What the call field in the QSO panel does on every key. */
  function type(call: string) {
    app.qso.call = call;
    app.callLocked = true;
    app.noteCallChange();
  }

  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('a typed call is looked up and fills name and QTH', async () => {
    const api = await withLookup();
    app.qso = { ...app.qso, call: '', name: '', qth: '' };
    for (const c of ['O', 'ON', 'ON4', 'ON4Z', 'ON4ZZ']) type(c);
    await vi.advanceTimersByTimeAsync(800);
    expect(api.lookupCalls).toEqual(['ON4ZZ']);
    expect(app.qso.name).toBe('Jan');
    expect(app.qso.qth).toBe('Gent');
  });

  it('the name of the station before is replaced, a typed name is not', async () => {
    const api = await withLookup();
    // The decoder filled HANS and BERLIN for DL1ABC (demo conversation).
    expect(app.qso.name).toBe('HANS');
    type('ON4ZZ');
    await vi.advanceTimersByTimeAsync(800);
    expect(app.qso.name).toBe('Jan');
    expect(app.qso.qth).toBe('Gent');
    // The operator types a name; another lookup of the same call leaves it.
    app.qso = { ...app.qso, name: 'JEF' };
    type('ON4ZZ/P');
    await vi.advanceTimersByTimeAsync(800);
    api.lookupAnswers['ON4ZZ/P'] = { ...api.lookupAnswers.ON4ZZ, call: 'ON4ZZ/P' };
    expect(app.qso.name).toBe('JEF');
  });

  it('the grey QRZ line belongs to the call in the panel only', async () => {
    await withLookup();
    type('ON4ZZ');
    await vi.advanceTimersByTimeAsync(800);
    expect(app.lookupNote?.call).toBe('ON4ZZ');
    type('G3ABC');
    expect(app.lookupNote).toBeNull();
  });
});

describe('practice in its own tab and radio controls (0.13.1)', () => {
  const on = (e: unknown[]) => (app as unknown as { onEvents(e: unknown[]): void }).onEvents(e);

  it('a paddle exercise opens a fresh Practice tab and leaves the QSO alone', async () => {
    await fresh();
    await app.requestPolicy('practice');
    const before = app.sessions.length;
    const qsoTab = app.cur.id;
    await app.paddleStart('chars');
    expect(app.sessions.length).toBe(before + 1);
    expect(app.cur.id).not.toBe(qsoTab);
    expect(app.cur.practice).toBe(true);
    expect(app.title(app.cur)).toMatch(/Practice|Oefenen/);
    expect(app.sessions.find((s) => s.id === qsoTab)?.qso.call).toBe('DL1ABC');
  });

  it('shows what the radio reports and warns when the volume is 0', async () => {
    await fresh();
    on([{ type: 'radio_controls', controls: { af_level: 0, break_in: 0, squelch: 255 } }]);
    expect(app.radioControls?.af_level).toBe(0);
    expect(app.radioControls?.squelch).toBe(255);
  });
});

describe('Koch method listening', () => {
  it('plays only chosen characters, scores per character and keeps the result', async () => {
    await fresh();
    await app.setKochCount(5);
    expect(app.settings!.practice.koch.chars).toBe('KMRSU');
    app.listenStart();
    const target = app.listenEx!.target;
    expect(target.replace(/ /g, '')).toMatch(/^[KMRSU]+$/);
    // Typed with one wrong character.
    const letters = [...target.replace(/ /g, '')];
    letters[0] = letters[0] === 'K' ? 'M' : 'K';
    app.listenEx!.typed = letters.join('').toLowerCase();
    await app.listenSubmit();
    expect(app.listenEx!.result!.accuracy).toBeCloseTo(19 / 20, 5);
    const k = app.settings!.practice.koch;
    expect(k.sessions).toHaveLength(1);
    expect(k.sessions[0].n).toBe(5);
    expect(k.stats.reduce((a, s) => a + s.errors, 0)).toBe(1);
    app.listenStop();
    expect(app.listenEx).toBeNull();
  });

  it('keeps the effective speed at or below the character speed', async () => {
    await fresh();
    await app.setKoch({ char_wpm: 18, eff_wpm: 30 });
    expect(app.settings!.practice.koch.eff_wpm).toBe(18);
  });
});

describe('fading switch', () => {
  it('is on by default, and the choice is kept', async () => {
    await fresh();
    expect(app.settings!.audio.fading).toBe(true);
    await app.setFading(false);
    expect(app.settings!.audio.fading).toBe(false);
  });
});

describe('DX cluster spots', () => {
  it('is off until the operator turns it on, and needs an own callsign', async () => {
    const api = await fresh([]);
    expect(app.clusterStatus.state).toBe('off');
    expect(app.settings!.cluster.enabled).toBe(false);
    expect(app.settings!.cluster.skimmer).toBe(true);
    expect(app.settings!.cluster.nodes[0].host).toBe('on0dxk.dyndns.org');
    await app.clusterConnect();
    expect(app.clusterStatus.state, 'no callsign, no login').toBe('off');
    expect(app.error).toBeTruthy();
    void api;
  });

  it('connects, receives spots and colors them by the logbook', async () => {
    const api = await fresh();
    await app.clusterConnect();
    api.flush();
    expect(app.clusterStatus.state).toBe('online');
    expect(app.clusterSpots.length).toBe(4);
    expect(app.clusterSpots.every((s) => s.status === 'new')).toBe(true);
    // Work a station that is on the list: its label turns to "worked" after the refresh.
    const spot = app.clusterSpots[0];
    app.selectCall(spot.call);
    app.entry.rst = '599';
    await app.logQso();
    await Promise.resolve();
    await app.clusterRefresh();
    expect(app.clusterSpots.find((s) => s.call === spot.call)?.status).toBe('worked');
    expect(app.clusterSpots.filter((s) => s.status === 'new').length).toBe(3);
  });

  it('a spot on another band of a worked station is "other_band"', async () => {
    const api = await fresh();
    const blank = ['rst_sent', 'rst_rcvd', 'nr_rcvd', 'extra_sent', 'extra_rcvd', 'name', 'qth', 'comment', 'contest', 'my_call', 'station_call', 'operator', 'my_grid', 'tx_pwr', 'my_rig', 'my_antenna', 'my_wwff', 'my_pota', 'my_sota', 'their_prog', 'their_ref', 'exported_to'];
    await api.logAdd({ ...Object.fromEntries(blank.map((k) => [k, ''])), id: 0, call: 'PA3ABC', band: '20m', freq_hz: 14_030_000, mode: 'CW', ts: Math.floor(Date.now() / 1000), nr_sent: 0, session: 1, test: false, deleted: false, activation_id: 0 } as never);
    await app.clusterConnect();
    api.flush();
    expect(app.clusterSpots.find((s) => s.call === 'PA3ABC')?.status).toBe('other_band');
  });

  it('merges a repeated spot of the same station instead of adding a second label', async () => {
    const api = await fresh();
    await app.clusterConnect();
    api.flush();
    const first = app.clusterSpots[0];
    api.emitNow([{ type: 'cluster_spots', spots: [{ ...first, snr: 30, hits: 2, freq_hz: first.freq_hz + 100 }] }]);
    expect(app.clusterSpots.length).toBe(4);
    expect(app.clusterSpots[0].snr).toBe(30);
    api.emitNow([{ type: 'cluster_spots', spots: [{ ...first, call: 'NEW1AA', freq_hz: first.freq_hz + 5000 }] }]);
    expect(app.clusterSpots.length).toBe(5);
  });

  it('disconnecting clears the spots, and the labels switch is remembered', async () => {
    const api = await fresh();
    await app.clusterConnect();
    api.flush();
    await app.setClusterLabels(false);
    expect(app.settings!.cluster.show_labels).toBe(false);
    await app.clusterDisconnect();
    expect(app.clusterStatus.state).toBe('off');
    expect(app.clusterSpots).toEqual([]);
  });

  it('only reading commands can be typed, and the answer shows in the window', async () => {
    const api = await fresh();
    await app.clusterConnect();
    api.flush();
    await app.clusterSendLine('sh/dx 5');
    api.flush();
    expect(app.clusterLines.some((l) => l.includes('sh/dx 5'))).toBe(true);
    app.error = null;
    await app.clusterSendLine('dx 7030 PA3ABC up');
    expect(app.error).toBeTruthy();
    expect(app.clusterLines.some((l) => l.includes('dx 7030'))).toBe(false);
  });

  it('tests the nodes and keeps the result', async () => {
    await fresh();
    await app.clusterTest();
    expect(app.clusterProbeResult.busy).toBe(false);
    expect(app.clusterProbeResult.list.length).toBe(8);
    expect(app.clusterProbeResult.list.some((r) => !r.ok)).toBe(true);
  });
});

describe('hole in the audio', () => {
  it('shows and clears the notch warning as the core reports it', async () => {
    const api = await fresh();
    expect(app.audioHole).toBeNull();
    api.emitNow([{ type: 'audio_hole', hz: 650 }]);
    expect(app.audioHole).toBe(650);
    api.emitNow([{ type: 'audio_hole', hz: null }]);
    expect(app.audioHole).toBeNull();
  });
});
