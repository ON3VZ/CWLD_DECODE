import type { LogQso } from './types';

export type SortKey = 'ts' | 'call' | 'band' | 'freq_hz' | 'mode' | 'name' | 'qth' | 'contest';

export interface Column {
  key: string;
  value: (q: LogQso) => string;
  sort?: SortKey;
  num?: boolean;
  mono?: boolean;
}

const date = (ts: number) => new Date(ts * 1000).toISOString().slice(0, 10);
const time = (ts: number) => new Date(ts * 1000).toISOString().slice(11, 16);
const join = (...v: string[]) => v.filter((x) => x && x.trim()).join(' ');

/** Where we listened during a QSO: its own frequency in split, else the transmit frequency. */
export function rxHz(q: LogQso): number {
  return q.freq_rx_hz || q.freq_hz;
}

/** True for a QSO made in split: we listened somewhere else than we transmitted. */
export function isSplit(q: LogQso): boolean {
  return !!q.freq_rx_hz && q.freq_rx_hz !== q.freq_hz;
}

/** Every column the logbook can show, in this order. */
export const COLUMNS: Column[] = [
  { key: 'date', value: (q) => date(q.ts), sort: 'ts', mono: true },
  { key: 'time', value: (q) => time(q.ts), mono: true },
  { key: 'call', value: (q) => q.call, sort: 'call', mono: true },
  { key: 'band', value: (q) => q.band, sort: 'band' },
  { key: 'freq', value: (q) => (q.freq_hz ? (q.freq_hz / 1000).toFixed(1) : ''), sort: 'freq_hz', num: true, mono: true },
  { key: 'freq_rx', value: (q) => (rxHz(q) ? (rxHz(q) / 1000).toFixed(1) : ''), num: true, mono: true },
  { key: 'mode', value: (q) => q.mode, sort: 'mode' },
  { key: 'rst_sent', value: (q) => q.rst_sent, mono: true },
  { key: 'rst_rcvd', value: (q) => q.rst_rcvd, mono: true },
  { key: 'exch_sent', value: (q) => join(q.nr_sent ? String(q.nr_sent) : '', q.extra_sent), mono: true },
  { key: 'exch_rcvd', value: (q) => join(q.nr_rcvd, q.extra_rcvd), mono: true },
  { key: 'name', value: (q) => q.name, sort: 'name' },
  { key: 'qth', value: (q) => q.qth, sort: 'qth' },
  { key: 'their_ref', value: (q) => join(q.their_prog, q.their_ref) },
  { key: 'comment', value: (q) => q.comment },
  { key: 'contest', value: (q) => (q.contest ? `${q.contest} #${q.session}` : ''), sort: 'contest' },
  { key: 'station', value: (q) => q.station_call || q.my_call, mono: true },
  { key: 'operator', value: (q) => q.operator, mono: true },
  { key: 'my_grid', value: (q) => q.my_grid, mono: true },
  { key: 'my_ref', value: (q) => join(q.my_wwff, q.my_pota, q.my_sota) },
  { key: 'power', value: (q) => q.tx_pwr },
  { key: 'rig', value: (q) => q.my_rig },
  { key: 'antenna', value: (q) => q.my_antenna },
  { key: 'exported', value: (q) => (q.exported_to.includes('adif') ? 'ADIF' : '') },
];

export const DEFAULT_COLUMNS = ['date', 'time', 'call', 'band', 'freq', 'freq_rx', 'mode', 'rst_sent', 'rst_rcvd', 'exch_rcvd', 'name', 'qth', 'their_ref', 'contest', 'station', 'exported'];

const STORE = 'cwld.logbook.columns';

/** The columns chosen earlier on this computer. Storage is only a convenience: it may fail. */
export function loadColumns(): string[] {
  try {
    const v = JSON.parse(localStorage.getItem(STORE) ?? 'null');
    if (Array.isArray(v) && v.every((x) => typeof x === 'string')) {
      const known = v.filter((k) => COLUMNS.some((c) => c.key === k));
      if (known.length) return known;
    }
  } catch {
    /* no storage: the default columns */
  }
  return [...DEFAULT_COLUMNS];
}

export function saveColumns(keys: string[]) {
  try {
    localStorage.setItem(STORE, JSON.stringify(keys));
  } catch {
    /* not kept, no harm */
  }
}

const BAND_ORDER = ['160m', '80m', '60m', '40m', '30m', '20m', '17m', '15m', '12m', '10m', '6m', '2m'];

/** Sorted copy. Equal values keep the newest first. */
export function sortRows(rows: LogQso[], key: SortKey, up: boolean): LogQso[] {
  const val = (q: LogQso): string | number => {
    if (key === 'band') {
      const i = BAND_ORDER.indexOf(q.band);
      return i < 0 ? 99 : i;
    }
    const v = q[key];
    return typeof v === 'number' ? v : String(v ?? '').toUpperCase();
  };
  const dir = up ? 1 : -1;
  return [...rows].sort((a, b) => {
    const x = val(a);
    const y = val(b);
    if (x < y) return -dir;
    if (x > y) return dir;
    return b.ts - a.ts || b.id - a.id;
  });
}

/** Counts for the line above the table. */
export function summary(rows: LogQso[]): { qsos: number; calls: number; bands: number } {
  return { qsos: rows.length, calls: new Set(rows.map((q) => q.call)).size, bands: new Set(rows.map((q) => q.band).filter(Boolean)).size };
}

/** ADIF band for a frequency in Hz, the same table as the core. Empty outside the amateur bands. */
export function bandOf(hz: number): string {
  const k = Math.floor(hz / 1000);
  const t: [number, number, string][] = [
    [1800, 2000, '160m'], [3500, 4000, '80m'], [5250, 5450, '60m'], [7000, 7300, '40m'], [10100, 10150, '30m'],
    [14000, 14350, '20m'], [18068, 18168, '17m'], [21000, 21450, '15m'], [24890, 24990, '12m'], [28000, 29700, '10m'],
    [50000, 54000, '6m'], [144000, 148000, '2m'],
  ];
  return t.find(([a, b]) => k >= a && k <= b)?.[2] ?? '';
}

/** "2026-11-12" + "14:05" or "14:05:30" (UTC) to seconds since 1970. NaN when it does not read as a date and time. */
export function parseUtc(day: string, time: string): number {
  const m = /^(\d{1,2}):(\d{2})(?::(\d{2}))?$/.exec(time.trim());
  if (!m || !/^\d{4}-\d{2}-\d{2}$/.test(day.trim()) || +m[1] > 23 || +m[2] > 59 || +(m[3] ?? 0) > 59) return NaN;
  const ms = Date.parse(`${day.trim()}T${m[1].padStart(2, '0')}:${m[2]}:${m[3] ?? '00'}Z`);
  return Number.isNaN(ms) ? NaN : ms / 1000;
}

/** Kilohertz as typed ("14025", "14025,5") to Hz. 0 when empty, NaN when it is not a number. */
export function parseKhz(text: string): number {
  const v = text.trim().replace(',', '.');
  if (!v) return 0;
  const n = Number(v);
  return Number.isFinite(n) && n >= 0 ? Math.round(n * 1000) : NaN;
}

/** One CSV field. A field with the separator, a quote or a line break gets quotes. A leading = + - @ is made harmless for spreadsheets. */
export function csvField(v: string, sep = ';'): string {
  let x = v;
  if (/^[=+@]/.test(x) || (/^-/.test(x) && !/^-?\d+([.,]\d+)?$/.test(x))) x = `'${x}`;
  return x.includes(sep) || /["\r\n]/.test(x) ? `"${x.replace(/"/g, '""')}"` : x;
}

/** The rows as CSV text, in the order and with the columns given. `head` gives the title of a column. Lines end with CRLF, the text starts with a BOM so Excel reads UTF-8. */
export function csvText(rows: LogQso[], keys: string[], head: (key: string) => string, sep = ';'): string {
  const cols = COLUMNS.filter((c) => keys.includes(c.key));
  const lines = [cols.map((c) => csvField(head(c.key), sep)).join(sep), ...rows.map((q) => cols.map((c) => csvField(c.value(q), sep)).join(sep))];
  return '\uFEFF' + lines.join('\r\n') + '\r\n';
}
