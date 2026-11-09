/**
 * The practice simulator: simulated stations that react to what you send. It runs in the
 * interface (it knows the contest profiles and the templates) and hands every transmission to
 * the core, which puts it on the practice band. Nothing here ever reaches the radio.
 *
 * Pure logic with a seeded random generator: the same seed and the same input give the same
 * stations, so the tests can follow a whole exercise.
 */
import type { ContestSettings, LogQso, PracticeSettings, PracticeSignal } from './types';
import { activeProfile, recvApplies, usesSerial } from './contest';

/** Small seeded generator (mulberry32). */
export class Rand {
  private s: number;
  constructor(seed: number) {
    this.s = seed >>> 0 || 1;
  }
  next(): number {
    let t = (this.s = (this.s + 0x6d2b79f5) >>> 0);
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  }
  range(a: number, b: number): number {
    return a + (b - a) * this.next();
  }
  int(a: number, b: number): number {
    return Math.floor(this.range(a, b + 1));
  }
  pick<T>(a: readonly T[]): T {
    return a[Math.floor(this.next() * a.length) % a.length];
  }
  chance(p: number): boolean {
    return this.next() < p;
  }
}

interface Country {
  pre: string[];
  digits: string;
  suffix: [number, number];
  cq: number;
  itu: number;
  weight: number;
  names: string[];
  qths: string[];
  /** ARRL DX: the state or province these stations send. */
  states?: string[];
}

// Zones per prefix area. A rough model for practice, not a reference.
const COUNTRIES: Country[] = [
  { pre: ['DL', 'DK', 'DJ', 'DF', 'DO'], digits: '123456789', suffix: [2, 3], cq: 14, itu: 28, weight: 14, names: ['HANS', 'PETER', 'KLAUS', 'UWE', 'JUERGEN', 'WOLF', 'ANDI', 'FRANK'], qths: ['BERLIN', 'HAMBURG', 'MUNICH', 'KOELN', 'BONN', 'ESSEN', 'AACHEN'] },
  { pre: ['PA', 'PD', 'PE'], digits: '0123456789', suffix: [2, 3], cq: 14, itu: 27, weight: 8, names: ['JAN', 'PIET', 'KEES', 'HENK', 'BART', 'GERT'], qths: ['UTRECHT', 'BREDA', 'DELFT', 'ZWOLLE', 'GOUDA', 'TILBURG'] },
  { pre: ['ON', 'OO', 'OT', 'OR'], digits: '3456789', suffix: [2, 3], cq: 14, itu: 27, weight: 7, names: ['JAN', 'LUC', 'MARC', 'DIRK', 'PATRICK', 'GEERT', 'ERIK'], qths: ['GENT', 'BRUGGE', 'LEUVEN', 'HASSELT', 'MECHELEN', 'NAMUR', 'LIEGE'] },
  { pre: ['F'], digits: '14568', suffix: [2, 3], cq: 14, itu: 27, weight: 7, names: ['JEAN', 'PIERRE', 'ALAIN', 'MICHEL', 'ERIC', 'PHIL'], qths: ['LYON', 'LILLE', 'NANTES', 'REIMS', 'DIJON', 'ROUEN'] },
  { pre: ['G', 'M', 'GM', 'GW'], digits: '0345', suffix: [3, 3], cq: 14, itu: 27, weight: 7, names: ['JOHN', 'DAVE', 'MIKE', 'STEVE', 'PAUL', 'IAN'], qths: ['LONDON', 'LEEDS', 'YORK', 'BRISTOL', 'OXFORD', 'CARDIFF'] },
  { pre: ['OK', 'OL'], digits: '12', suffix: [3, 3], cq: 15, itu: 28, weight: 4, names: ['PETR', 'JIRI', 'PAVEL', 'KAREL'], qths: ['PRAHA', 'BRNO', 'PLZEN'] },
  { pre: ['SP', 'SQ', 'SO'], digits: '123456789', suffix: [2, 3], cq: 15, itu: 28, weight: 5, names: ['JUREK', 'ADAM', 'MAREK', 'TOMEK'], qths: ['KRAKOW', 'WARSZAWA', 'GDANSK', 'POZNAN'] },
  { pre: ['I', 'IK', 'IZ', 'IW'], digits: '0123456789', suffix: [3, 3], cq: 15, itu: 28, weight: 5, names: ['MARIO', 'LUCA', 'GIANNI', 'FRANCO'], qths: ['ROMA', 'MILANO', 'TORINO', 'NAPOLI'] },
  { pre: ['EA', 'EB', 'EC'], digits: '1234578', suffix: [2, 3], cq: 14, itu: 37, weight: 4, names: ['JOSE', 'PACO', 'JUAN', 'TONI'], qths: ['MADRID', 'SEVILLA', 'VALENCIA', 'BILBAO'] },
  { pre: ['OH'], digits: '1234569', suffix: [2, 3], cq: 15, itu: 18, weight: 3, names: ['JUHA', 'MATTI', 'KARI'], qths: ['HELSINKI', 'OULU', 'TURKU'] },
  { pre: ['SM', 'SA'], digits: '0234567', suffix: [3, 3], cq: 14, itu: 18, weight: 3, names: ['LARS', 'OLLE', 'ANDERS'], qths: ['STOCKHOLM', 'MALMO', 'UPPSALA'] },
  { pre: ['OZ', 'OU'], digits: '1234579', suffix: [2, 3], cq: 14, itu: 18, weight: 2, names: ['OLE', 'BENT', 'JENS'], qths: ['ODENSE', 'AARHUS'] },
  { pre: ['HA', 'HG'], digits: '135678', suffix: [2, 3], cq: 15, itu: 28, weight: 2, names: ['LACI', 'JOE', 'GABOR'], qths: ['BUDAPEST', 'GYOR'] },
  { pre: ['S5'], digits: '0123456789', suffix: [2, 2], cq: 15, itu: 28, weight: 2, names: ['TONE', 'MARKO'], qths: ['LJUBLJANA'] },
  { pre: ['9A'], digits: '123456789', suffix: [2, 3], cq: 15, itu: 28, weight: 2, names: ['IVAN', 'ZELJKO'], qths: ['ZAGREB', 'SPLIT'] },
  { pre: ['UA', 'RA', 'RW'], digits: '1346', suffix: [2, 3], cq: 16, itu: 29, weight: 4, names: ['SERGEY', 'ALEX', 'VLAD', 'OLEG'], qths: ['MOSCOW', 'KAZAN', 'TULA'] },
  { pre: ['UR', 'UT'], digits: '0123456789', suffix: [2, 3], cq: 16, itu: 29, weight: 2, names: ['YURI', 'IGOR'], qths: ['KYIV', 'LVIV'] },
  { pre: ['LY'], digits: '12345', suffix: [2, 3], cq: 15, itu: 29, weight: 1, names: ['VYTAS', 'ROMAS'], qths: ['VILNIUS'] },
  { pre: ['OE'], digits: '13568', suffix: [3, 3], cq: 15, itu: 28, weight: 2, names: ['FRANZ', 'SEPP'], qths: ['WIEN', 'GRAZ', 'LINZ'] },
  { pre: ['HB9'], digits: '', suffix: [2, 3], cq: 14, itu: 28, weight: 2, names: ['BEAT', 'URS', 'HANS'], qths: ['BERN', 'ZURICH', 'BASEL'] },
  { pre: ['CT'], digits: '127', suffix: [3, 3], cq: 14, itu: 37, weight: 1, names: ['JOAO', 'RUI'], qths: ['LISBOA', 'PORTO'] },
  { pre: ['EI'], digits: '2345789', suffix: [2, 3], cq: 14, itu: 27, weight: 1, names: ['SEAN', 'PAT'], qths: ['DUBLIN', 'CORK'] },
  { pre: ['K', 'W', 'N', 'AA', 'KB'], digits: '0123456789', suffix: [2, 3], cq: 5, itu: 8, weight: 4, names: ['BOB', 'JIM', 'TOM', 'BILL', 'RICK', 'DAN'], qths: ['BOSTON', 'OHIO', 'TEXAS', 'MAINE', 'IOWA'], states: ['MA', 'NY', 'OH', 'PA', 'CT', 'NJ', 'VA', 'NC', 'FL', 'TX', 'CA', 'IL', 'MI', 'WI', 'MN', 'CO'] },
  { pre: ['VE', 'VA'], digits: '1236', suffix: [2, 3], cq: 4, itu: 4, weight: 1, names: ['GORD', 'RON'], qths: ['TORONTO', 'OTTAWA'], states: ['ON', 'QC', 'NS', 'BC', 'AB', 'MB'] },
  { pre: ['JA', 'JH', 'JR'], digits: '1234567', suffix: [3, 3], cq: 25, itu: 45, weight: 1, names: ['TAKA', 'HIRO', 'KEN'], qths: ['TOKYO', 'OSAKA'] },
  { pre: ['VK'], digits: '2345', suffix: [3, 3], cq: 30, itu: 59, weight: 1, names: ['BRUCE', 'COL'], qths: ['SYDNEY', 'PERTH'] },
  { pre: ['PY'], digits: '1234', suffix: [2, 3], cq: 11, itu: 15, weight: 1, names: ['CARLOS', 'PAULO'], qths: ['SAO PAULO', 'RIO'] },
];

/** Section codes used for simulated Belgian callers. Illustrative only, not the official UBA list. */
export const SIM_SECTIONS = ['ANT', 'WLD', 'MCL', 'LIR', 'BRC', 'GBN', 'TLS', 'DST', 'ODE'];

const LETTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';

export type StationState = 'cq' | 'calling' | 'picked' | 'info' | 'exch' | 'done' | 'gone';

export interface SimStation {
  call: string;
  name: string;
  qth: string;
  wpm: number;
  tone: number;
  snr: number;
  qsb: number;
  jitter: number;
  /** The report this station gives you, as keyed (5NN or 579). */
  rst: string;
  /** Contest: serial number (without leading zeros) and the extra field it sends. */
  nr: string;
  extra: string;
  state: StationState;
  tries: number;
  /** Times it may still call again without being answered. */
  patience: number;
  /** The QSO was completed from the station's side (it heard your TU or 73). */
  worked: boolean;
  /** It was ever heard on the band. */
  heard: boolean;
  lastText: string;
  nextAt: number;
}

/** One transmission of a simulated station, ready for the core. */
export interface SimTx extends PracticeSignal {
  call: string;
}

export interface SimOptions {
  cfg: PracticeSettings;
  myCall: string;
  myName: string;
  /** The pitch the decoder listens to: stations sit around it. */
  toneHz: number;
  contest?: ContestSettings;
  seed: number;
}

/** SNR in 500 Hz per strength choice. */
export const STRENGTH_DB: Record<PracticeSettings['strength'], [number, number]> = { weak: [-2, 8], medium: [8, 18], strong: [18, 30] };

/** Edit distance between two calls. */
export function distance(a: string, b: string): number {
  const d: number[][] = Array.from({ length: a.length + 1 }, (_, i) => [i, ...Array(b.length).fill(0)]);
  for (let j = 1; j <= b.length; j++) d[0][j] = j;
  for (let i = 1; i <= a.length; i++)
    for (let j = 1; j <= b.length; j++) d[i][j] = Math.min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
  return d[a.length][b.length];
}

const words = (text: string) => text.toUpperCase().split(/\s+/).filter(Boolean);
const isRst = (w: string) => /^(5[1-9][1-9N]|5NN|[1-5][1-9][1-9])$/.test(w);
const asksAgain = (ws: string[]) => ws.some((w) => w === '?' || w === 'AGN' || w === 'AGN?');
const asksNr = (ws: string[]) => ws.some((w) => w === 'NR?' || w === 'NR');
/** Leading zeros of a serial as the station keys them: plain, padded, or cut (T for 0). */
function keyedNr(rand: Rand, nr: number): string {
  const r = rand.next();
  if (r < 0.4) return String(nr);
  const padded = String(nr).padStart(3, '0');
  return r < 0.7 ? padded : padded.replace(/0/g, 'T').replace(/9/g, 'N');
}
const cleanNr = (s: string) => s.toUpperCase().replace(/T/g, '0').replace(/N/g, '9').replace(/^0+(?=\d)/, '');
const sameRst = (a: string, b: string) => a.toUpperCase().replace(/N/g, '9') === b.toUpperCase().replace(/N/g, '9');

export interface ScoreLine {
  /** What you logged. */
  call: string;
  /** The real call of the station, when it differs or when nothing matched. */
  truth: string | null;
  ok: boolean;
  /** What was wrong: call, nr, extra, rst, name, qth. */
  problems: string[];
}

export interface Score {
  exercise: PracticeSettings['exercise'];
  logged: number;
  correct: number;
  busted: number;
  wrongExchange: number;
  /** Stations you completed (they heard your TU or 73) but did not log. */
  missed: string[];
  minutes: number;
  /** Correct QSOs per hour. */
  rate: number;
  lines: ScoreLine[];
}

export class PracticeSim {
  readonly rand: Rand;
  readonly opt: SimOptions;
  stations: SimStation[] = [];
  /** Calls of stations transmitting now. */
  onAir = new Set<string>();
  /** The station of the QSO exercise. */
  current: SimStation | null = null;
  startedAt = 0;
  /** Last moment something happened on the band (you or a station). */
  lastActivity = 0;
  private myBase: string;

  constructor(opt: SimOptions) {
    this.opt = opt;
    this.rand = new Rand(opt.seed);
    this.myBase = opt.myCall.toUpperCase().split('/')[0];
  }

  get exercise() {
    return this.opt.cfg.exercise;
  }

  private country(): Country {
    const prof = activeProfile(this.opt.contest);
    let pool = COUNTRIES;
    if (this.exercise === 'contest' && prof) {
      if (prof.recv?.name === 'STATE') pool = COUNTRIES.filter((c) => c.states);
      else if (prof.id === 'uba-on-cw' || prof.id === 'uba-winter-cw') pool = this.rand.chance(0.85) ? COUNTRIES.filter((c) => c.pre.includes('ON')) : COUNTRIES;
    }
    const total = pool.reduce((a, c) => a + c.weight, 0);
    let r = this.rand.next() * total;
    for (const c of pool) {
      r -= c.weight;
      if (r <= 0) return c;
    }
    return pool[0];
  }

  /** A new random station, not one already on the band and never your own call. */
  newStation(state: StationState, now: number): SimStation {
    const { cfg } = this.opt;
    for (;;) {
      const c = this.country();
      const digit = c.digits ? this.rand.pick([...c.digits]) : '';
      const n = this.rand.int(c.suffix[0], c.suffix[1]);
      let suf = '';
      for (let i = 0; i < n; i++) suf += this.rand.pick([...LETTERS]);
      const call = `${this.rand.pick(c.pre)}${digit}${suf}`;
      if (call === this.myBase || this.stations.some((s) => s.call === call)) continue;
      const [lo, hi] = STRENGTH_DB[cfg.strength] ?? STRENGTH_DB.medium;
      const spread = this.exercise === 'contest' ? 120 : 40;
      const wMin = Math.min(cfg.wpm_min, cfg.wpm_max);
      const wMax = Math.max(cfg.wpm_min, cfg.wpm_max);
      const st: SimStation = {
        call,
        name: this.rand.pick(c.names),
        qth: this.rand.pick(c.qths),
        wpm: Math.round(this.rand.range(wMin, wMax)),
        tone: Math.round(this.opt.toneHz + this.rand.range(-spread, spread)),
        snr: Math.round(this.rand.range(lo, hi)),
        qsb: cfg.qsb ? this.rand.range(0.3, 0.8) : 0,
        jitter: this.exercise === 'contest' ? this.rand.range(0, 0.03) : this.rand.range(0, 0.08),
        rst: this.exercise === 'contest' ? (this.rand.chance(0.85) ? '5NN' : '599') : this.rand.pick(['599', '579', '589', '559', '5NN']),
        nr: '',
        extra: '',
        state,
        tries: 0,
        patience: this.rand.int(1, 4),
        worked: false,
        heard: false,
        lastText: '',
        nextAt: now,
      };
      this.exchangeFor(st, c);
      this.stations.push(st);
      return st;
    }
  }

  private exchangeFor(st: SimStation, c: Country) {
    if (this.exercise !== 'contest') return;
    const contest = this.opt.contest;
    if (usesSerial(contest)) st.nr = String(1 + Math.floor(this.rand.next() ** 2 * 1200));
    const recv = activeProfile(contest)?.recv ?? null;
    if (!recv || !recvApplies(recv, st.call)) return;
    if (recv.name === 'ZONE') st.extra = String(c.cq);
    else if (recv.name === 'ITU') st.extra = String(c.itu);
    else if (recv.name === 'STATE') st.extra = this.rand.pick(c.states ?? ['MA']);
    else if (recv.name === 'SEC') st.extra = this.rand.pick(SIM_SECTIONS);
    else if (recv.kind === 'choice' && recv.choices.length) st.extra = this.rand.pick(recv.choices).toUpperCase();
    else if (recv.kind === 'number') st.extra = String(this.rand.int(1, 40));
    else st.extra = this.rand.pick([...LETTERS]) + this.rand.pick([...LETTERS]);
  }

  /** The exchange the station keys: report, number and extra field. */
  private exchangeText(st: SimStation): string {
    const parts = [st.rst];
    if (st.nr) parts.push(keyedNr(this.rand, Number(st.nr)));
    if (st.extra) parts.push(st.extra);
    return parts.join(' ');
  }

  private tx(st: SimStation, text: string, delay: number): SimTx {
    st.lastText = text;
    st.heard = true;
    this.onAir.add(st.call);
    return { call: st.call, text, wpm: st.wpm, tone_hz: st.tone, snr_db: st.snr, qsb_depth: st.qsb, jitter: st.jitter, delay_s: delay, seed: Math.floor(this.rand.next() * 1e9) + 1 };
  }

  private my(): string {
    return this.opt.myCall.toUpperCase();
  }

  // ---- The QSO exercises ----

  private cqText(st: SimStation): string {
    return this.rand.chance(0.5) ? `CQ CQ DE ${st.call} ${st.call} K` : `CQ CQ CQ DE ${st.call} ${st.call} ${st.call} K`;
  }

  private infoText(st: SimStation, answering: boolean): string {
    const me = this.my();
    const greet = this.rand.pick(['GM', 'GA', 'GE']);
    const intro = answering ? `R ${greet} ${this.opt.myName ? this.opt.myName + ' ' : ''}TNX FER RPT` : `${greet} TNX FER CALL`;
    return `${me} DE ${st.call} ${intro} UR RST ${st.rst} ${st.rst} NAME ${st.name} ${st.name} QTH ${st.qth} ${st.qth} HW? ${me} DE ${st.call} K`;
  }

  private finalText(st: SimStation): string {
    const me = this.my();
    return `${me} DE ${st.call} R TNX FER QSO ${this.opt.myName ? this.opt.myName + ' ' : ''}73 GL ${me} DE ${st.call} TU E E`;
  }

  /** Starts the exercise. Returns what the band should key first. */
  start(now: number): SimTx[] {
    this.startedAt = now;
    this.lastActivity = now;
    if (this.exercise === 'they_cq') return this.nextCq(now, 1);
    return [];
  }

  private nextCq(now: number, delay: number): SimTx[] {
    const st = this.newStation('cq', now);
    this.current = st;
    return [this.tx(st, this.cqText(st), delay)];
  }

  /** You sent `text` (your transmission is over). The stations react. */
  userSent(text: string, now: number): SimTx[] {
    this.lastActivity = now;
    const ws = words(text);
    if (!ws.length) return [];
    return this.exercise === 'contest' ? this.contestReact(ws, now) : this.qsoReact(ws, now);
  }

  private qsoReact(ws: string[], now: number): SimTx[] {
    const st = this.current;
    const isCq = ws.includes('CQ');
    if (!st || st.state === 'done' || st.state === 'gone') {
      if (this.exercise === 'you_cq' && isCq) {
        const caller = this.newStation('calling', now);
        this.current = caller;
        caller.tries = 1;
        const text = this.rand.chance(0.5) ? `${this.my()} DE ${caller.call} ${caller.call} K` : `${caller.call} ${caller.call}`;
        return [this.tx(caller, text, this.rand.range(1.2, 2.8))];
      }
      return [];
    }
    if (this.onAir.has(st.call)) return [];
    const hasHis = ws.some((w) => w.replace(/\?$/, '') === st.call);
    const hasMine = ws.some((w) => w.split('/')[0] === this.myBase);
    const delay = this.rand.range(0.8, 1.8);
    if (st.state === 'cq') {
      if (hasHis || hasMine) {
        st.state = 'info';
        return [this.tx(st, this.infoText(st, false), delay)];
      }
      return [];
    }
    if (st.state === 'calling') {
      if (isCq) return [this.tx(st, `${st.call} ${st.call}`, delay)];
      if (hasHis) {
        st.state = 'info';
        return [this.tx(st, this.infoText(st, true), delay)];
      }
      const near = ws.some((w) => w.length >= 2 && (st.call.includes(w.replace(/\?$/, '')) || distance(w.replace(/\?$/, ''), st.call) <= 2));
      if (near || asksAgain(ws)) return [this.tx(st, `${st.call} ${st.call} K`, delay)];
      return [];
    }
    if (st.state === 'info') {
      if (asksAgain(ws) && ws.length <= 3) return [this.tx(st, st.lastText, delay)];
      if (ws.includes('NAME?')) return [this.tx(st, `NAME ${st.name} ${st.name}`, delay)];
      if (ws.includes('QTH?')) return [this.tx(st, `QTH ${st.qth} ${st.qth}`, delay)];
      if (ws.includes('RST?')) return [this.tx(st, `RST ${st.rst} ${st.rst}`, delay)];
      st.state = 'exch';
      st.worked = true;
      return [this.tx(st, this.finalText(st), delay)];
    }
    return [];
  }

  // ---- The contest exercise (Run) ----

  private contestReact(ws: string[], now: number): SimTx[] {
    const out: SimTx[] = [];
    const active = this.stations.filter((s) => s.state === 'calling' || s.state === 'picked' || s.state === 'exch');
    const inQso = active.find((s) => s.state === 'exch') ?? active.find((s) => s.state === 'picked') ?? null;
    const plain = ws.map((w) => w.replace(/\?+$/, ''));
    const isCq = ws.includes('CQ') || ws.includes('TEST') || ws.includes('QRZ') || ws.includes('QRZ?');
    const hasRst = ws.some(isRst);
    const delay = () => this.rand.range(0.3, 1.2);

    // Your TU after his exchange closes the QSO.
    let closed = false;
    if (inQso?.state === 'exch' && (ws.includes('TU') || ws.includes('R') || isCq) && !(plain.includes(inQso.call) && hasRst)) {
      inQso.state = 'done';
      inQso.worked = true;
      closed = true;
    }
    if (!closed && inQso?.state === 'exch' && !this.onAir.has(inQso.call)) {
      if (asksNr(ws) && inQso.nr) return [this.tx(inQso, `${keyedNr(this.rand, Number(inQso.nr))} ${keyedNr(this.rand, Number(inQso.nr))}`, delay())];
      if (asksAgain(ws) && !plain.some((w) => w.length > 2 && /\d/.test(w))) return [this.tx(inQso, this.exchangeText(inQso), delay())];
    }

    // A call: exact, close, or a part of one.
    const callers = this.stations.filter((s) => s.state === 'calling' || s.state === 'picked' || (s.state === 'exch' && !closed));
    const exact = callers.find((s) => plain.includes(s.call));
    if (exact && !closed) {
      for (const s of callers) if (s !== exact && s.state !== 'calling') s.state = 'calling';
      if (this.onAir.has(exact.call)) return [];
      if (hasRst) {
        exact.state = 'exch';
        const lead = this.rand.pick(['TU ', 'R ', '', '']);
        return [this.tx(exact, `${lead}${this.exchangeText(exact)}`, delay())];
      }
      exact.state = 'picked';
      if (ws.some((w) => w.endsWith('?'))) return [this.tx(exact, `${exact.call} ${exact.call}`, delay())];
      return [];
    }
    const callWords = plain.filter((w) => w.length >= 3 && /\d/.test(w) && /[A-Z]/.test(w) && w.split('/')[0] !== this.myBase);
    if (!closed && callWords.length) {
      let best: SimStation | null = null;
      let bestD = 3;
      for (const s of callers)
        for (const w of callWords) {
          const d = distance(w, s.call);
          if (d < bestD) {
            bestD = d;
            best = s;
          }
        }
      if (best && !this.onAir.has(best.call)) {
        // A wrong call: the station corrects it, with its exchange when you sent yours.
        if (hasRst) {
          best.state = 'exch';
          return [this.tx(best, `${best.call} ${best.call} ${this.exchangeText(best)}`, delay())];
        }
        best.state = 'picked';
        return [this.tx(best, `${best.call} ${best.call}`, delay())];
      }
    }
    const parts = plain.filter((w) => w.length >= 2 && !isRst(w) && w.split('/')[0] !== this.myBase && !['CQ', 'TEST', 'TU', 'QRZ', 'AGN', 'NR', 'DE'].includes(w));
    if (!closed && !isCq && parts.length) {
      for (const s of callers.filter((x) => x.state === 'calling' && !this.onAir.has(x.call)))
        if (parts.some((p) => s.call.includes(p))) out.push(this.tx(s, `${s.call} ${s.call}`, delay()));
      if (out.length) return out;
    }
    if (!closed && !isCq && asksAgain(ws)) {
      for (const s of callers.filter((x) => x.state === 'calling' && !this.onAir.has(x.call))) out.push(this.tx(s, s.call, delay()));
      return out;
    }
    if (isCq || closed) return this.invite(now);
    return out;
  }

  /** After a CQ or a TU: waiting callers call again, new ones join. */
  private invite(now: number): SimTx[] {
    const out: SimTx[] = [];
    const max = Math.max(1, Math.min(8, this.opt.cfg.pileup));
    for (const s of this.stations) {
      if (s.state === 'picked') s.state = 'calling';
      if (s.state !== 'calling') continue;
      if (s.patience <= 0) {
        s.state = 'gone';
        continue;
      }
      s.patience--;
      if (!this.onAir.has(s.call) && this.rand.chance(0.85)) out.push(this.tx(s, this.callText(s), this.rand.range(0.2, 1.8)));
    }
    const waiting = this.stations.filter((s) => s.state === 'calling').length;
    // Sometimes nobody new: a quiet moment on the band.
    const fresh = this.rand.chance(0.12) ? 0 : this.rand.int(waiting ? 0 : 1, Math.max(0, max - waiting));
    for (let i = 0; i < fresh; i++) {
      const s = this.newStation('calling', now);
      out.push(this.tx(s, this.callText(s), this.rand.range(0.2, 1.8)));
    }
    return out;
  }

  private callText(s: SimStation): string {
    s.tries++;
    return this.rand.chance(0.25) ? `${s.call} ${s.call}` : s.call;
  }

  /** A transmission of `call` ended. */
  done(call: string, now: number): SimTx[] {
    this.onAir.delete(call);
    this.lastActivity = now;
    const st = this.stations.find((s) => s.call === call);
    if (!st) return [];
    if (this.exercise !== 'contest') {
      if (st.state === 'cq') st.nextAt = now + this.rand.range(3, 5);
      if (st.state === 'calling') st.nextAt = now + this.rand.range(5, 7);
      if (st.state === 'exch') {
        st.state = 'done';
        if (this.exercise === 'they_cq') st.nextAt = now + this.rand.range(5, 8);
      }
    }
    return [];
  }

  /** Called about every half second: stations that waited long enough act again. */
  tick(now: number): SimTx[] {
    if (this.onAir.size) return [];
    if (this.exercise === 'contest') {
      const calling = this.stations.filter((s) => s.state === 'calling');
      if (calling.length && now - this.lastActivity >= 5) {
        this.lastActivity = now;
        const out: SimTx[] = [];
        for (const s of calling) {
          if (s.patience <= 0) {
            s.state = 'gone';
            continue;
          }
          s.patience--;
          out.push(this.tx(s, this.callText(s), this.rand.range(0.1, 1.0)));
        }
        return out;
      }
      return [];
    }
    const st = this.current;
    if (!st) return [];
    if (st.state === 'cq' && now >= st.nextAt) {
      st.tries++;
      if (st.tries > 6) {
        st.state = 'gone';
        return this.nextCq(now, this.rand.range(2, 4));
      }
      return [this.tx(st, this.cqText(st), 0.2)];
    }
    if (st.state === 'calling' && now >= st.nextAt) {
      st.tries++;
      if (st.tries > 3) {
        st.state = 'gone';
        this.current = null;
        return [];
      }
      return [this.tx(st, `${st.call} ${st.call} K`, 0.2)];
    }
    if (st.state === 'done' && this.exercise === 'they_cq' && now >= st.nextAt) return this.nextCq(now, 0.5);
    if (st.state === 'done' && this.exercise === 'you_cq') this.current = null;
    return [];
  }

  /** Compares what you logged with what the stations really sent. */
  score(logged: LogQso[], now: number): Score {
    const heard = this.stations.filter((s) => s.heard);
    const lines: ScoreLine[] = [];
    let correct = 0;
    let busted = 0;
    let wrongExchange = 0;
    const loggedCalls = new Set<string>();
    for (const q of [...logged].sort((a, b) => a.ts - b.ts)) {
      const call = q.call.toUpperCase();
      loggedCalls.add(call);
      let st = heard.find((s) => s.call === call) ?? null;
      const problems: string[] = [];
      if (!st) {
        let bestD = 3;
        for (const s of heard) {
          const d = distance(call, s.call);
          if (d < bestD) {
            bestD = d;
            st = s;
          }
        }
        problems.push('call');
        busted++;
      }
      if (st) {
        if (st.call !== call) loggedCalls.add(st.call);
        if (this.exercise === 'contest') {
          if (st.nr && cleanNr(q.nr_rcvd) !== st.nr) problems.push('nr');
          if (st.extra && q.extra_rcvd.toUpperCase() !== st.extra) problems.push('extra');
        } else {
          if (q.rst_rcvd && !sameRst(q.rst_rcvd, st.rst)) problems.push('rst');
          if (q.name && q.name.toUpperCase() !== st.name) problems.push('name');
          if (q.qth && q.qth.toUpperCase() !== st.qth) problems.push('qth');
        }
      }
      const exchBad = problems.some((p) => p !== 'call');
      if (exchBad && !problems.includes('call')) wrongExchange++;
      const ok = problems.length === 0;
      if (ok) correct++;
      lines.push({ call, truth: st && st.call !== call ? st.call : null, ok, problems });
    }
    const missed = this.stations.filter((s) => s.worked && !loggedCalls.has(s.call)).map((s) => s.call);
    const minutes = Math.max(0, (now - this.startedAt) / 60);
    return { exercise: this.exercise, logged: logged.length, correct, busted, wrongExchange, missed, minutes, rate: minutes > 0 ? Math.round((correct * 60) / minutes) : 0, lines };
  }
}
