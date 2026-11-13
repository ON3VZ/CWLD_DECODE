/**
 * Country, direction and distance for a callsign.
 *
 * The country comes from the prefix table in `dxcc-data.ts` (longest prefix wins). Direction and
 * distance are computed from the own locator to the locator of the other station when QRZ.com gave
 * one, else to the middle of the country. Nothing here sends or stores anything: it is only shown.
 */
import { PREFIX_TABLE } from './dxcc-data';

export interface Entity {
  name: string;
  continent: string;
  lat: number;
  lon: number;
}

export interface CallPlace {
  entity: Entity;
  /** The prefix of the table that matched. */
  prefix: string;
}

const EARTH_KM = 6371;
const SUFFIXES = new Set(['P', 'M', 'A', 'AM', 'QRP', 'QRPP', 'LH', 'B', 'T', 'R', 'J', 'S', 'D', 'X']);
/** Maritime and aeronautical mobile belong to no country. */
const NO_COUNTRY = new Set(['MM', 'AM']);

/** Expands `[A-R]`, `[890]` and `?` in one table token into plain prefixes. */
export function expandToken(token: string): string[] {
  let out = [''];
  for (let i = 0; i < token.length; i++) {
    const c = token[i];
    let choices: string[];
    if (c === '[') {
      const end = token.indexOf(']', i);
      if (end < 0) throw new Error(`unclosed [ in prefix ${token}`);
      choices = [];
      const body = token.slice(i + 1, end);
      for (let j = 0; j < body.length; j++) {
        if (body[j + 1] === '-' && j + 2 < body.length) {
          for (let k = body.charCodeAt(j); k <= body.charCodeAt(j + 2); k++) choices.push(String.fromCharCode(k));
          j += 2;
        } else choices.push(body[j]);
      }
      i = end;
    } else if (c === '?') {
      choices = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('');
    } else choices = [c];
    out = out.flatMap((p) => choices.map((x) => p + x));
  }
  return out;
}

export interface Tables {
  prefixes: Map<string, Entity>;
  exact: Map<string, Entity>;
  maxLen: number;
}

/** Reads the table text. Throws on a broken line, so a mistake in the data shows in the tests. */
export function parseTable(text: string): Tables {
  const prefixes = new Map<string, Entity>();
  const exact = new Map<string, Entity>();
  let maxLen = 1;
  for (const raw of text.split('\n')) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) continue;
    const [name, continent, lat, lon, list] = line.split('|');
    const e: Entity = { name, continent, lat: Number(lat), lon: Number(lon) };
    if (!name || !continent || !list || !Number.isFinite(e.lat) || !Number.isFinite(e.lon)) throw new Error(`bad prefix line: ${line}`);
    for (const tok of list.split(',')) {
      const t = tok.trim();
      if (!t) continue;
      if (t.startsWith('=')) {
        exact.set(t.slice(1), e);
        continue;
      }
      for (const p of expandToken(t)) {
        prefixes.set(p, e);
        maxLen = Math.max(maxLen, p.length);
      }
    }
  }
  return { prefixes, exact, maxLen };
}

let tables: Tables | null = null;
function table(): Tables {
  return (tables ??= parseTable(PREFIX_TABLE));
}

/** Longest prefix of `s` in the table. */
function longest(s: string, tb: Tables): CallPlace | null {
  for (let n = Math.min(tb.maxLen, s.length); n >= 1; n--) {
    const p = s.slice(0, n);
    const e = tb.prefixes.get(p);
    if (e) return { entity: e, prefix: p };
  }
  return null;
}

const FULL_CALL = /^(?:[A-Z]{1,2}|[0-9][A-Z]|[A-Z][0-9])[0-9][A-Z]{1,4}$/;

/**
 * Country of a callsign, or null when unknown (or maritime and aeronautical mobile).
 * Portable forms: `DL/ON3VZ` and `ON3VZ/DL` use the prefix part, `ON3VZ/P`, `/M`, `/QRP` and a
 * single digit or letter behind the slash change nothing.
 */
export function lookupCall(call: string, tb: Tables = table()): CallPlace | null {
  const up = call.trim().toUpperCase();
  if (!/^[A-Z0-9/]+$/.test(up)) return null;
  const hit = tb.exact.get(up);
  if (hit) return { entity: hit, prefix: up };
  const parts = up.split('/').filter(Boolean);
  if (!parts.length) return null;
  if (parts.some((p) => NO_COUNTRY.has(p))) return null;
  // Drop endings that say nothing about the country.
  const core = parts.filter((p, i) => !(i > 0 && (SUFFIXES.has(p) || /^[0-9]$/.test(p) || /^[A-Z]$/.test(p))));
  if (core.length === 1) return longest(core[0], tb);
  // Two parts: the short one that is not a full callsign is the prefix of the country visited.
  const [a, b] = core;
  const full = (s: string) => FULL_CALL.test(s);
  if (full(a) && !full(b) && b.length <= 5) return longest(b, tb) ?? longest(a, tb);
  if (!full(a) && full(b) && a.length <= 5) return longest(a, tb) ?? longest(b, tb);
  return longest(a.length <= b.length ? a : b, tb) ?? longest(a.length > b.length ? a : b, tb);
}

// ---- Locators, direction and distance ----

export interface LatLon {
  lat: number;
  lon: number;
}

/** The middle of a Maidenhead locator of four or six characters, or null when it is not one. */
export function locatorToLatLon(loc: string): LatLon | null {
  const s = loc.trim().toUpperCase();
  const m = /^([A-R]{2})([0-9]{2})([A-X]{2})?$/.exec(s);
  if (!m) return null;
  const A = 'A'.charCodeAt(0);
  let lon = (m[1].charCodeAt(0) - A) * 20 - 180 + Number(m[2][0]) * 2;
  let lat = (m[1].charCodeAt(1) - A) * 10 - 90 + Number(m[2][1]);
  if (m[3]) {
    lon += (m[3].charCodeAt(0) - A) * (2 / 24) + 1 / 24;
    lat += (m[3].charCodeAt(1) - A) * (1 / 24) + 1 / 48;
  } else {
    lon += 1;
    lat += 0.5;
  }
  return { lat, lon };
}

const rad = (d: number) => (d * Math.PI) / 180;
const deg = (r: number) => (r * 180) / Math.PI;

/** Great circle distance in km. */
export function distanceKm(a: LatLon, b: LatLon): number {
  const dLat = rad(b.lat - a.lat);
  const dLon = rad(b.lon - a.lon);
  const h = Math.sin(dLat / 2) ** 2 + Math.cos(rad(a.lat)) * Math.cos(rad(b.lat)) * Math.sin(dLon / 2) ** 2;
  return 2 * EARTH_KM * Math.asin(Math.min(1, Math.sqrt(h)));
}

/** Initial bearing from a to b in degrees, 0 to 359 (short path). */
export function bearingDeg(a: LatLon, b: LatLon): number {
  const y = Math.sin(rad(b.lon - a.lon)) * Math.cos(rad(b.lat));
  const x = Math.cos(rad(a.lat)) * Math.sin(rad(b.lat)) - Math.sin(rad(a.lat)) * Math.cos(rad(b.lat)) * Math.cos(rad(b.lon - a.lon));
  return (deg(Math.atan2(y, x)) + 360) % 360;
}

export interface PlaceInfo {
  country: string;
  continent: string;
  prefix: string;
  /** Short path, degrees and km. Null without an own locator. */
  bearing: number | null;
  longPath: number | null;
  km: number | null;
  /** `grid`: to the locator QRZ.com gave. `country`: to the middle of the country, so rough. */
  basis: 'grid' | 'country' | null;
  /** The other station is in the country of the own call: a distance to the middle says nothing. */
  home: boolean;
}

/**
 * Everything to show for a callsign. `myGrid` is the own locator, `theirGrid` the locator QRZ.com
 * gave for the other station (if any), `myCall` is used to see "same country".
 */
export function describeCall(call: string, myGrid: string, theirGrid = '', myCall = ''): PlaceInfo | null {
  const hit = lookupCall(call);
  if (!hit) return null;
  const me = locatorToLatLon(myGrid);
  const grid = locatorToLatLon(theirGrid);
  const mine = myCall ? lookupCall(myCall) : null;
  const home = !!mine && mine.entity.name === hit.entity.name;
  const info: PlaceInfo = { country: hit.entity.name, continent: hit.entity.continent, prefix: hit.prefix, bearing: null, longPath: null, km: null, basis: null, home };
  if (!me) return info;
  const target: LatLon | null = grid ?? (home ? null : { lat: hit.entity.lat, lon: hit.entity.lon });
  if (!target) return info;
  const km = distanceKm(me, target);
  if (km < 1) return info;
  const b = bearingDeg(me, target);
  info.km = Math.round(km);
  info.bearing = Math.round(b) % 360;
  info.longPath = Math.round((b + 180) % 360) % 360;
  info.basis = grid ? 'grid' : 'country';
  return info;
}

/** Distance as shown: whole km, with a thin space for thousands (`5 140`). */
export function formatKm(km: number): string {
  return String(km).replace(/\B(?=(\d{3})+(?!\d))/g, ' ');
}
