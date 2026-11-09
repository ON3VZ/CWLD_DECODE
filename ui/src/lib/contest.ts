import type { ContestProfile, ContestSettings, ContestVar, RecvField } from './types';

/** Calls of Belgian stations: the prefixes ON, OO, OP, OQ, OR, OS and OT. */
export const BELGIAN_PREFIXES = ['ON', 'OO', 'OP', 'OQ', 'OR', 'OS', 'OT'];

/** Names the app fills in itself. An own variable cannot use them. */
export const RESERVED_VARS = ['CALL', 'MYCALL', 'NAME', 'QTH', 'RST', 'NR', 'MYNAME', 'MYQTH'];

const NAME_RE = /^[A-Z][A-Z0-9_]{0,15}$/;

export function validVarName(name: string): boolean {
  return NAME_RE.test(name) && !RESERVED_VARS.includes(name);
}

const be = (label: string, template: string) => ({ label, template });
const FROM_BE = 'i18n:contest.sent.be';
const ABROAD = 'i18n:contest.sent.abroad';

function uba(id: string, name: string, calls: string[], note: string): ContestProfile {
  return {
    id,
    name,
    note,
    calls,
    sent: [be(FROM_BE, '{CALL} {RST} {NR} {SEC}'), be(ABROAD, '{CALL} {RST} {NR}')],
    recv: { name: 'SEC', label: 'UBA section', kind: 'text', choices: [], only_prefixes: BELGIAN_PREFIXES },
    cut_numbers: false,
  };
}

/** Built-in profiles. They cannot be changed: copy one to make an own version. Rules checked on 7 November 2026, see docs. */
export const BUILTIN_PROFILES: ContestProfile[] = [
  uba('uba-dx-cw', 'UBA DX Contest CW', ['TEST UBA'], 'i18n:contest.note.uba'),
  uba('uba-on-cw', 'UBA ON Contest CW', ['CQ ON CONTEST'], 'i18n:contest.note.uba'),
  uba('uba-spring-cw', 'UBA Spring Contest CW', ['CQ UBA CONTEST'], 'i18n:contest.note.uba_old'),
  {
    id: 'uba-winter-cw',
    name: 'UBA Low Band Winter Contest CW',
    note: 'i18n:contest.note.winter',
    calls: ['CQ UBA WINTERCONTEST'],
    sent: [be(FROM_BE, '{CALL} {RST} {SEC}'), be(ABROAD, '{CALL} {RST} {NR}')],
    recv: null,
    cut_numbers: false,
  },
  {
    id: 'cqww-cw',
    name: 'CQ WW DX Contest CW',
    note: 'i18n:contest.note.cqww',
    calls: ['CQ TEST'],
    sent: [be('i18n:contest.sent.default', '{CALL} {RST} {ZONE}')],
    recv: { name: 'ZONE', label: 'CQ zone', kind: 'number', choices: [], only_prefixes: [] },
    cut_numbers: false,
  },
  {
    id: 'cqwpx-cw',
    name: 'CQ WPX Contest CW',
    note: '',
    calls: ['CQ TEST'],
    sent: [be('i18n:contest.sent.default', '{CALL} {RST} {NR}')],
    recv: null,
    cut_numbers: false,
  },
  {
    id: 'arrl-dx-cw',
    name: 'ARRL International DX Contest CW',
    note: 'i18n:contest.note.arrl',
    calls: ['CQ TEST'],
    sent: [be('i18n:contest.sent.default', '{CALL} {RST} {PWR}')],
    recv: { name: 'STATE', label: 'State or province', kind: 'text', choices: [], only_prefixes: [] },
    cut_numbers: false,
  },
  {
    id: 'iaru-hf',
    name: 'IARU HF World Championship',
    note: 'i18n:contest.note.iaru',
    calls: ['CQ TEST'],
    sent: [be('i18n:contest.sent.default', '{CALL} {RST} {ITU}')],
    recv: { name: 'ITU', label: 'ITU zone or society', kind: 'text', choices: [], only_prefixes: [] },
    cut_numbers: false,
  },
  {
    id: 'wae-cw',
    name: 'WAE DX Contest CW',
    note: 'i18n:contest.note.wae',
    calls: ['CQ TEST'],
    sent: [be('i18n:contest.sent.default', '{CALL} {RST} {NR}')],
    recv: null,
    cut_numbers: false,
  },
  {
    id: 'blank',
    name: 'i18n:contest.blank',
    note: '',
    calls: ['CQ TEST'],
    sent: [be('i18n:contest.sent.default', '{CALL} {RST} {NR}')],
    recv: null,
    cut_numbers: false,
  },
];

export const BUILTIN_IDS = new Set(BUILTIN_PROFILES.map((p) => p.id));

export function isBuiltin(id: string): boolean {
  return BUILTIN_IDS.has(id);
}

export function allProfiles(c: ContestSettings): ContestProfile[] {
  return [...BUILTIN_PROFILES, ...c.profiles];
}

export function activeProfile(c: ContestSettings | undefined): ContestProfile | null {
  if (!c || !c.active) return null;
  return allProfiles(c).find((p) => p.id === c.active) ?? null;
}

/** The sent template that is in use: the chosen one, or the first when the number is out of range. */
export function activeSent(c: ContestSettings | undefined): string | null {
  const p = activeProfile(c);
  if (!p || !p.sent.length) return null;
  return (p.sent[c!.sent_index] ?? p.sent[0]).template;
}

/** Whether the active exchange template uses the serial number. */
export function usesSerial(c: ContestSettings | undefined): boolean {
  const tpl = activeSent(c);
  return tpl === null ? true : tpl.includes('{NR}');
}

/** Cut numbers: 0 becomes T and 9 becomes N. */
export function cutNumber(text: string): string {
  return text.replace(/0/g, 'T').replace(/9/g, 'N');
}

/** Own variables as name and value. Invalid and reserved names are left out. */
export function ownVars(c: ContestSettings | undefined): Record<string, string> {
  const out: Record<string, string> = {};
  for (const v of c?.vars ?? []) {
    const name = v.name.trim().toUpperCase();
    if (validVarName(name)) out[name] = v.value.trim().toUpperCase();
  }
  return out;
}

/** Whether the call belongs to a station the extra field applies to. */
export function recvApplies(field: RecvField | null, call: string): boolean {
  if (!field) return false;
  if (!field.only_prefixes.length) return true;
  const c = call.trim().toUpperCase();
  return field.only_prefixes.some((p) => c.startsWith(p));
}

export function newProfileId(existing: ContestProfile[]): string {
  let n = existing.length + 1;
  while (existing.some((p) => p.id === `own-${n}`)) n++;
  return `own-${n}`;
}

/** Text that is either plain or `i18n:key`, resolved with `tr`. */
export function plain(text: string, tr: (key: string) => string): string {
  return text.startsWith('i18n:') ? tr(text.slice(5)) : text;
}

/** A copy that can be edited: new id, own name, readable labels and note (no keys). */
export function copyProfile(p: ContestProfile, name: string, existing: ContestProfile[], tr: (key: string) => string): ContestProfile {
  const c = structuredClone(p);
  c.id = newProfileId(existing);
  c.name = name;
  c.note = plain(c.note, tr);
  c.sent = c.sent.map((s) => ({ ...s, label: plain(s.label, tr) }));
  return c;
}

/** Cleans a profile coming from a file. Returns null when it is not usable. */
export function cleanProfile(raw: unknown): ContestProfile | null {
  if (!raw || typeof raw !== 'object') return null;
  const r = raw as Record<string, unknown>;
  const str = (v: unknown) => (typeof v === 'string' ? v : '');
  const name = str(r.name).trim();
  if (!name) return null;
  const sent = Array.isArray(r.sent)
    ? r.sent
        .map((x) => (x && typeof x === 'object' ? { label: str((x as Record<string, unknown>).label), template: str((x as Record<string, unknown>).template) } : null))
        .filter((x): x is { label: string; template: string } => !!x && x.template.trim() !== '')
    : [];
  if (!sent.length) return null;
  let recv: RecvField | null = null;
  if (r.recv && typeof r.recv === 'object') {
    const f = r.recv as Record<string, unknown>;
    const kind = f.kind === 'number' || f.kind === 'choice' ? f.kind : 'text';
    const strs = (v: unknown) => (Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string').map((x) => x.trim().toUpperCase()).filter(Boolean) : []);
    const fname = str(f.name).trim().toUpperCase();
    if (fname) recv = { name: fname, label: str(f.label).trim() || fname, kind, choices: strs(f.choices), only_prefixes: strs(f.only_prefixes) };
  }
  const calls = Array.isArray(r.calls) ? r.calls.filter((x): x is string => typeof x === 'string').map((x) => x.trim().toUpperCase()).filter(Boolean) : [];
  return { id: str(r.id), name, note: str(r.note), calls, sent: sent.map((s) => ({ label: s.label.trim() || 'Exchange', template: s.template.trim().toUpperCase() })), recv, cut_numbers: r.cut_numbers === true };
}

export interface ContestExport {
  cwld_contest: 1;
  profiles: ContestProfile[];
  vars: ContestVar[];
}

/** The text of an export file: own profiles and own variables. */
export function exportText(c: ContestSettings, onlyId?: string): string {
  const profiles = onlyId ? allProfiles(c).filter((p) => p.id === onlyId) : c.profiles;
  const data: ContestExport = { cwld_contest: 1, profiles: profiles.map((p) => ({ ...p })), vars: onlyId ? [] : c.vars };
  return JSON.stringify(data, null, 2) + '\n';
}

export interface ImportResult {
  profiles: ContestProfile[];
  vars: ContestVar[];
  skipped: number;
}

/** Reads an export file. Throws an Error with a short code when the text is not an export. */
export function parseImport(text: string): ImportResult {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    throw new Error('not_json');
  }
  if (!raw || typeof raw !== 'object' || (raw as Record<string, unknown>).cwld_contest !== 1) throw new Error('not_export');
  const r = raw as Record<string, unknown>;
  const list = Array.isArray(r.profiles) ? r.profiles : [];
  const profiles: ContestProfile[] = [];
  let skipped = 0;
  for (const x of list) {
    const p = cleanProfile(x);
    if (p) profiles.push(p);
    else skipped++;
  }
  const vars: ContestVar[] = [];
  for (const v of Array.isArray(r.vars) ? r.vars : []) {
    if (v && typeof v === 'object') {
      const o = v as Record<string, unknown>;
      const name = typeof o.name === 'string' ? o.name.trim().toUpperCase() : '';
      if (validVarName(name)) vars.push({ name, value: typeof o.value === 'string' ? o.value.trim().toUpperCase() : '' });
      else skipped++;
    }
  }
  return { profiles, vars, skipped };
}

/** Merges an import into the settings. A profile with the same name gets a new id and the suffix " (2)". */
export function mergeImport(c: ContestSettings, imp: ImportResult, replaceSameName: boolean): ContestSettings {
  const profiles = [...c.profiles];
  for (const p of imp.profiles) {
    const i = profiles.findIndex((x) => x.name === p.name);
    if (i >= 0 && replaceSameName) {
      profiles[i] = { ...p, id: profiles[i].id };
    } else {
      let name = p.name;
      let n = 2;
      while (profiles.some((x) => x.name === name) || BUILTIN_PROFILES.some((x) => x.name === name)) name = `${p.name} (${n++})`;
      profiles.push({ ...p, name, id: newProfileId([...BUILTIN_PROFILES, ...profiles]) });
    }
  }
  const vars = [...c.vars];
  for (const v of imp.vars) {
    const i = vars.findIndex((x) => x.name.toUpperCase() === v.name);
    if (i >= 0) vars[i] = v;
    else vars.push(v);
  }
  return { ...c, profiles, vars };
}

/**
 * Suggested ADIF CONTEST_ID per built-in profile. These are the values as the author remembers them
 * and have NOT been checked against the official ADIF list yet, so the export dialog shows the value
 * in an editable field. UBA ON, Spring, Winter and WAE are left empty on purpose.
 */
export const ADIF_CONTEST_ID_SUGGESTIONS: Record<string, string> = {
  'uba-dx-cw': 'UBA-DX-CW',
  'cqww-cw': 'CQ-WW-CW',
  'cqwpx-cw': 'CQ-WPX-CW',
  'arrl-dx-cw': 'ARRL-DX-CW',
  'iaru-hf': 'IARU-HF',
};
