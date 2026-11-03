import { beforeEach, describe, expect, it } from 'vitest';
import { app } from './app.svelte';
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
    const ok = await app.connectRadio({ kind: 'icom', port: '', civ_address: 0xb6, baud: 115200, pipeline: false });
    expect(ok).toBe(false);
    expect(app.error).toMatch(/port|poort/i);
    expect(app.status.connected).toBe(true);
  });

  it('connects and disconnects', async () => {
    await fresh();
    expect(await app.connectRadio({ kind: 'qmx', port: 'COM6', civ_address: 0xb6, baud: 115200, pipeline: false })).toBe(true);
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
