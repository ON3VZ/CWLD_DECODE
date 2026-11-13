import type { LogQso } from './types';

/** ADIF band name for a frequency in Hz (same table as the Rust logbook). */
export function bandOf(hz: number): string {
  const k = Math.floor(hz / 1000);
  const t: [number, number, string][] = [
    [1800, 2000, '160m'], [3500, 4000, '80m'], [5250, 5450, '60m'], [7000, 7300, '40m'], [10100, 10150, '30m'],
    [14000, 14350, '20m'], [18068, 18168, '17m'], [21000, 21450, '15m'], [24890, 24990, '12m'], [28000, 29700, '10m'],
    [50000, 54000, '6m'], [144000, 148000, '2m'],
  ];
  return t.find(([a, b]) => k >= a && k <= b)?.[2] ?? '';
}

const f = (name: string, v: string) => {
  const s = v.trim().replace(/[^\x20-\x7e]/g, '?');
  return s ? `<${name}:${s.length}>${s}` : '';
};

/** A small ADIF writer for the browser stand-in. The real one is in Rust (crates/cwld-log). */
export function adifText(qsos: LogQso[], contestId: string): string {
  const rows = [...qsos].sort((a, b) => a.ts - b.ts || a.id - b.id).map((q) => {
    const d = new Date(q.ts * 1000).toISOString();
    return [
      f('CALL', q.call), f('QSO_DATE', d.slice(0, 10).replace(/-/g, '')), f('TIME_ON', d.slice(11, 19).replace(/:/g, '')),
      f('BAND', q.band), q.freq_hz ? f('FREQ', (q.freq_hz / 1e6).toFixed(6)) : '', q.freq_rx_hz && q.freq_rx_hz !== q.freq_hz ? f('FREQ_RX', (q.freq_rx_hz / 1e6).toFixed(6)) : '', f('MODE', q.mode || 'CW'),
      f('RST_SENT', q.rst_sent), f('RST_RCVD', q.rst_rcvd), q.nr_sent > 0 ? f('STX', String(q.nr_sent)) : '', f('STX_STRING', q.extra_sent),
      /^\d+$/.test(q.nr_rcvd) ? f('SRX', String(Number(q.nr_rcvd))) : '', f('SRX_STRING', q.extra_rcvd),
      q.contest ? f('CONTEST_ID', contestId) : '', f('STATION_CALLSIGN', q.my_call), f('OPERATOR', q.my_call), f('NAME', q.name), f('QTH', q.qth), f('COMMENT', q.comment),
      '<EOR>',
    ].join('');
  });
  return `CWLD ADIF export\n${f('ADIF_VER', '3.1.4')}${f('PROGRAMID', 'CWLD')}<EOH>\n${rows.join('\n')}${rows.length ? '\n' : ''}`;
}
