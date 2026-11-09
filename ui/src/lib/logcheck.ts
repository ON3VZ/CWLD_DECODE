import type { Activation, CheckIssue, LogQso } from './types';

/** A Maidenhead locator of six characters, such as JO20QE. */
export function validGrid(g: string): boolean {
  return /^[A-R]{2}[0-9]{2}[A-X]{2}$/i.test(g.trim());
}

/** A WWFF reference: two capital letters, FF, a dash and four digits, such as ONFF-0599. */
export function validWwffRef(r: string): boolean {
  return /^[A-Z]{2}FF-[0-9]{4}$/.test(r.trim());
}

/**
 * The same checks as the Rust logbook (crates/cwld-log/src/check.rs), for the browser stand-in.
 * The real check runs in Rust; this keeps the demo and the tests honest.
 */
export function checkExport(qsos: LogQso[], acts: Activation[], now: number): CheckIssue[] {
  const out: CheckIssue[] = [];
  const add = (severity: CheckIssue['severity'], code: string, id: number, detail = '') => out.push({ severity, code, id, detail });
  if (!qsos.length) return [{ severity: 'error', code: 'empty', id: 0, detail: '' }];
  for (const q of qsos) {
    const onAir = q.station_call || q.my_call;
    if (!onAir) add('error', 'no_station_call', q.id, q.call);
    if (!q.call || !q.band || !q.mode || q.ts <= 0) add('error', 'missing_basic', q.id, q.call);
    if (!q.rst_sent || !q.rst_rcvd) add('error', 'missing_rst', q.id, q.call);
    const portable = onAir.includes('/') || q.activation_id !== 0;
    if (!q.my_grid) add(portable ? 'error' : 'warning', 'no_grid', q.id, q.call);
    else if (!validGrid(q.my_grid)) add('error', 'bad_grid', q.id, q.my_grid);
    if (q.my_wwff && !validWwffRef(q.my_wwff)) add('error', 'bad_wwff_ref', q.id, q.my_wwff);
    if (q.their_prog === 'WWFF' && q.their_ref && !validWwffRef(q.their_ref)) add('error', 'bad_wwff_ref', q.id, q.their_ref);
    const a = acts.find((x) => x.id === q.activation_id);
    if (a) {
      const end = a.ended_ts === 0 ? now : a.ended_ts;
      if (q.ts < a.started_ts - 60 || q.ts > end + 60) add('error', 'outside_activation', q.id, q.call);
    }
  }
  const grids = [...new Set(qsos.map((q) => q.my_grid).filter(Boolean))];
  if (grids.length > 1) add('warning', 'several_grids', 0, grids.join(' '));
  const calls = [...new Set(qsos.map((q) => q.station_call || q.my_call).filter(Boolean))];
  if (calls.length > 1) add('warning', 'several_station_calls', 0, calls.join(' '));
  const seen = new Set<string>();
  for (const q of [...qsos].sort((a, b) => a.ts - b.ts || a.id - b.id)) {
    if (q.contest) continue;
    const key = `${q.activation_id}|${q.call}|${q.band}|${q.mode}`;
    if (seen.has(key)) add('warning', 'dupe', q.id, q.call);
    seen.add(key);
  }
  return out;
}
