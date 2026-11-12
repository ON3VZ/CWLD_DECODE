/**
 * Texts behind the chips and function keys.
 *
 * Every chip and key is a "slot" with a built-in text. The operator can change the text and the
 * name of every slot, also for the built-in contest profiles. Only the changes are stored
 * (`Settings.send_texts`), so "Restore default" is simply forgetting the change.
 *
 * Slot ids:
 *   chip.<id>                        a chip outside contest mode (built-in or own)
 *   pileup.F1 .. pileup.F4           the pile-up keys
 *   contest.<profile>.<run|sp>.F1..F8  contest keys per profile (`none` without a profile), as N1MM keeps
 *                                      function keys per contest
 */
import type { ContestSettings, OwnChip, SendTexts } from './types';
import { defaultChips, pileupChips } from './demo';
import { contestKeys, type ContestKey, type RunMode } from './contestlog';
import { activeProfile, ownVars, RESERVED_VARS } from './contest';

export const EMPTY_TEXTS: SendTexts = { texts: {}, labels: {}, own_chips: [], chip_order: [], cq_pause_s: 6, cq_max: 10, paddle_wait: true };

export const CQ_PAUSE_MIN = 2;
export const CQ_PAUSE_MAX = 60;
export const CQ_MAX_LIMIT = 30;
/** Longest text one slot may hold. Longer texts are cut when saved. */
export const TEMPLATE_MAX = 240;
export const LABEL_MAX = 24;

/** Settings from an older file may lack the block; fill in what is missing. */
export function textsOf(s: { send_texts?: Partial<SendTexts> } | null | undefined): SendTexts {
  const t = s?.send_texts ?? {};
  return {
    texts: t.texts ?? {},
    labels: t.labels ?? {},
    own_chips: t.own_chips ?? [],
    chip_order: t.chip_order ?? [],
    cq_pause_s: clamp(t.cq_pause_s ?? EMPTY_TEXTS.cq_pause_s, CQ_PAUSE_MIN, CQ_PAUSE_MAX),
    cq_max: clamp(t.cq_max ?? EMPTY_TEXTS.cq_max, 1, CQ_MAX_LIMIT),
    paddle_wait: t.paddle_wait ?? true,
  };
}

function clamp(v: number, lo: number, hi: number): number {
  if (!Number.isFinite(v)) return lo;
  return Math.min(hi, Math.max(lo, Math.round(v)));
}

export type SlotGroup = 'chip' | 'pileup' | 'contest';

export interface Slot {
  id: string;
  group: SlotGroup;
  /** Function key shown on the button, if any. */
  fkey?: string;
  /** Built-in name: an i18n key. Empty for own chips. */
  labelKey: string;
  /** Name the operator gave, or the name of an own chip. Empty: use `labelKey`. */
  label: string;
  /** Text that is sent (with variables). */
  template: string;
  /** Built-in text, for "Restore default". Empty for own chips. */
  def: string;
  /** The text differs from the built-in text (or the name does). */
  changed: boolean;
  own: boolean;
  /** Contest key that logs the QSO after sending (TU). */
  log?: boolean;
}

/** The name to show on a button. */
export function slotLabel(s: Slot, tr: (key: string) => string): string {
  return s.label || (s.labelKey ? tr(s.labelKey) : s.id);
}

function apply(id: string, group: SlotGroup, labelKey: string, def: string, t: SendTexts, extra: Partial<Slot> = {}): Slot {
  const tpl = t.texts[id];
  const label = t.labels[id] ?? '';
  return {
    id,
    group,
    labelKey,
    label,
    template: tpl ?? def,
    def,
    changed: (tpl !== undefined && tpl !== def) || label !== '',
    own: false,
    ...extra,
  };
}

/** The chips outside contest mode: built-in ones and own ones, in the operator's order. */
export function chipSlots(t: SendTexts): Slot[] {
  const builtIn = defaultChips.map((c) => apply(`chip.${c.id}`, 'chip', c.key, c.template, t));
  const own = t.own_chips.map(
    (c): Slot => ({ id: `chip.${c.id}`, group: 'chip', labelKey: '', label: c.label, template: c.template, def: '', changed: false, own: true }),
  );
  return orderChips([...builtIn, ...own], t.chip_order);
}

/** Puts the slots in the given order; slots that are not named keep their place after the named ones. */
export function orderChips(slots: Slot[], order: string[]): Slot[] {
  const pos = new Map(order.map((id, i) => [id, i]));
  return slots
    .map((s, i) => ({ s, i }))
    .sort((a, b) => (pos.get(a.s.id) ?? order.length + a.i) - (pos.get(b.s.id) ?? order.length + b.i))
    .map((x) => x.s);
}

export function pileupSlots(t: SendTexts): Slot[] {
  return pileupChips.map((c) => apply(`pileup.${c.fkey}`, 'pileup', c.key, c.template, t, { fkey: c.fkey }));
}

/**
 * Slot id of a contest key. When the profile has more than one exchange (from Belgium, from
 * abroad), F2 has a slot per exchange, so the choice in the contest bar keeps working.
 */
export function contestSlotId(profile: string, run: RunMode, key: string, sent?: number): string {
  return `contest.${profile || 'none'}.${run}.${key}${sent !== undefined ? `.s${sent}` : ''}`;
}

/** The contest keys F1 to F8 for the active profile and mode, with the operator's changes. */
export function contestSlots(run: RunMode, c: ContestSettings | undefined, t: SendTexts): Array<Slot & ContestKey> {
  const p = activeProfile(c);
  const profile = p?.id ?? '';
  const sent = p && p.sent.length > 1 ? Math.min(c?.sent_index ?? 0, p.sent.length - 1) : undefined;
  return contestKeys(run, c).map((k) => {
    const id = contestSlotId(profile, run, k.key, k.key === 'F2' ? sent : undefined);
    return { ...k, ...apply(id, 'contest', k.label, k.template, t, { fkey: k.key, log: k.log }) };
  });
}

/** The text of a slot after a change. Saving the built-in text again removes the change. */
export function setSlot(t: SendTexts, slot: Slot, template: string, label: string): SendTexts {
  const tpl = cleanTemplate(template);
  const name = label.trim().slice(0, LABEL_MAX);
  if (slot.own) {
    const id = slot.id.slice('chip.'.length);
    return { ...t, own_chips: t.own_chips.map((c) => (c.id === id ? { ...c, template: tpl, label: name || c.label } : c)) };
  }
  const texts = { ...t.texts };
  const labels = { ...t.labels };
  if (tpl === slot.def || tpl === '') delete texts[slot.id];
  else texts[slot.id] = tpl;
  const builtInName = name === '' || name === slot.labelKey;
  if (builtInName) delete labels[slot.id];
  else labels[slot.id] = name;
  return { ...t, texts, labels };
}

/** Forgets the change of one slot. */
export function restoreSlot(t: SendTexts, id: string): SendTexts {
  const texts = { ...t.texts };
  const labels = { ...t.labels };
  delete texts[id];
  delete labels[id];
  return { ...t, texts, labels };
}

/** Forgets every change of one group (own chips and the chip order stay unless asked). */
export function restoreGroup(t: SendTexts, prefix: string): SendTexts {
  const keep = (o: Record<string, string>) => Object.fromEntries(Object.entries(o).filter(([k]) => !k.startsWith(prefix)));
  return { ...t, texts: keep(t.texts), labels: keep(t.labels) };
}

export function newOwnChip(t: SendTexts, label: string, template: string): { texts: SendTexts; id: string } {
  let n = t.own_chips.length + 1;
  while (t.own_chips.some((c) => c.id === `own-${n}`)) n++;
  const chip: OwnChip = { id: `own-${n}`, label: label.trim().slice(0, LABEL_MAX) || `Chip ${n}`, template: cleanTemplate(template) };
  return { texts: { ...t, own_chips: [...t.own_chips, chip] }, id: `chip.${chip.id}` };
}

export function removeOwnChip(t: SendTexts, slotId: string): SendTexts {
  const id = slotId.slice('chip.'.length);
  return { ...t, own_chips: t.own_chips.filter((c) => c.id !== id), chip_order: t.chip_order.filter((x) => x !== slotId) };
}

/** Moves a chip one place up (-1) or down (+1). Stores the full order from then on. */
export function moveChip(t: SendTexts, slotId: string, dir: -1 | 1): SendTexts {
  const ids = chipSlots(t).map((s) => s.id);
  const i = ids.indexOf(slotId);
  const j = i + dir;
  if (i < 0 || j < 0 || j >= ids.length) return t;
  [ids[i], ids[j]] = [ids[j], ids[i]];
  return { ...t, chip_order: ids };
}

/** Capitals, one space between words, no line breaks, not too long. */
export function cleanTemplate(text: string): string {
  return text
    .replace(/\s+/g, ' ')
    .trim()
    .toUpperCase()
    .slice(0, TEMPLATE_MAX);
}

/** Names that can be used in a text: the built-in ones and the own contest variables. */
export function knownVars(c: ContestSettings | undefined): string[] {
  return [...RESERVED_VARS, ...Object.keys(ownVars(c))];
}

const SENDABLE = /^[A-Z0-9.,?/=+\-@ ]$/;
const PROSIGNS = ['AR', 'AS', 'BT', 'HH', 'SK', 'KN', 'BK', 'SN'];

export interface TemplateCheck {
  /** `{NAME}` that CWLD does not know. Sending would stop with "Missing". */
  unknownVars: string[];
  /** Characters the radio cannot send; the keyer leaves them out. */
  badChars: string[];
  /** `<XX>` that is not a known prosign. */
  badProsigns: string[];
  /** A `{` without `}` or the other way round. */
  braces: boolean;
}

/** Checks a text before it is saved. Nothing here blocks saving; the editor shows it as a warning. */
export function checkTemplate(template: string, known: string[]): TemplateCheck {
  const unknownVars: string[] = [];
  const badChars: string[] = [];
  const badProsigns: string[] = [];
  const tpl = template.toUpperCase();
  for (const m of tpl.matchAll(/\{([A-Z0-9_]+)\}/g)) if (!known.includes(m[1]) && !unknownVars.includes(m[1])) unknownVars.push(m[1]);
  for (const m of tpl.matchAll(/<([^>]*)>/g)) if (!PROSIGNS.includes(m[1]) && !badProsigns.includes(m[1])) badProsigns.push(m[1]);
  const rest = tpl.replace(/\{[A-Z0-9_]+\}/g, ' ').replace(/<[^>]*>/g, ' ');
  const braces = /[{}]/.test(rest);
  for (const ch of rest) if (!SENDABLE.test(ch) && !/[{}<>\s]/.test(ch) && !badChars.includes(ch)) badChars.push(ch);
  return { unknownVars, badChars, badProsigns, braces };
}

export function checkOk(c: TemplateCheck): boolean {
  return !c.unknownVars.length && !c.badChars.length && !c.badProsigns.length && !c.braces;
}

export interface PreviewPart {
  text: string;
  /** plain text, a filled-in variable, or a variable without a value. */
  kind: 'plain' | 'var' | 'missing';
}

/** The text as it would be sent now, in parts, so the editor can mark the variables. */
export function previewParts(template: string, vars: Record<string, string | undefined>): PreviewPart[] {
  const parts: PreviewPart[] = [];
  let last = 0;
  const tpl = template.toUpperCase();
  for (const m of tpl.matchAll(/\{([A-Z0-9_]+)\}/g)) {
    if (m.index! > last) parts.push({ text: tpl.slice(last, m.index), kind: 'plain' });
    const v = vars[m[1]];
    parts.push(v && v.trim() ? { text: v, kind: 'var' } : { text: m[0], kind: 'missing' });
    last = m.index! + m[0].length;
  }
  if (last < tpl.length) parts.push({ text: tpl.slice(last), kind: 'plain' });
  return parts;
}

/** The text that CQ repeat sends: F1 in contest Run mode, else the CQ chip. Null when there is none. */
export function cqSlot(contest: boolean, run: RunMode, c: ContestSettings | undefined, t: SendTexts): Slot | null {
  if (contest) return run === 'run' ? contestSlots('run', c, t)[0] : null;
  return chipSlots(t).find((s) => s.id === 'chip.cq') ?? null;
}
