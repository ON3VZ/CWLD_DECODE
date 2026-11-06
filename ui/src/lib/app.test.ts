import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { app, SETTLE_MS } from './app.svelte';
import { MockApi } from './mockApi';

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
