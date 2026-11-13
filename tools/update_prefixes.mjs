#!/usr/bin/env node
/**
 * Brings the prefix table of CWLD up to date from a Country Files list (`cty.dat`, the format that
 * AD1C publishes at country-files.com, the same list that most logging programs read).
 *
 *   node tools/update_prefixes.mjs path/to/cty.dat            writes ui/src/lib/dxcc-data.ts
 *   node tools/update_prefixes.mjs cty.dat --exact            also keeps the exact callsigns (a much bigger file)
 *   node tools/update_prefixes.mjs cty.dat --out other.ts     writes somewhere else
 *
 * Check the licence and the terms of the list you download before you put the result in the project.
 * Entities whose main prefix starts with `*` (only for the WAE award, not for DXCC) are skipped.
 * Overrides inside a prefix, such as (14)[27]<51/-4>, are dropped: CWLD only needs the country.
 * In cty.dat the longitude is positive to the WEST. CWLD wants east positive, so the sign is turned.
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { dirname, resolve } from 'node:path';

const CONT = new Set(['EU', 'AS', 'AF', 'NA', 'SA', 'OC']);

/** Converts the text of a cty.dat into the lines of the CWLD table. */
export function convertCty(text, { exact = false } = {}) {
  // An entity is one header line `Name: CQ: ITU: CONT: lat: lon: tz: prefix:` and then the aliases up to `;`.
  const records = text.replace(/\r/g, '').split(';');
  const lines = [];
  for (const rec of records) {
    const body = rec.trim();
    if (!body) continue;
    const nl = body.indexOf('\n');
    if (nl < 0) continue;
    const head = body.slice(0, nl).split(':').map((s) => s.trim());
    if (head.length < 8) continue;
    const [name, , , cont, lat, lon, , primary] = head;
    if (!CONT.has(cont) || primary.startsWith('*')) continue;
    const aliases = body.slice(nl + 1).split(/[,\s]+/).filter(Boolean);
    const list = new Set();
    for (const a of aliases) {
      const isExact = a.startsWith('=');
      let tok = (isExact ? a.slice(1) : a).replace(/\([^)]*\)|\[[^\]]*\]|<[^>]*>|\{[^}]*\}|~[^~]*~/g, '');
      if (!tok || tok.includes('/')) continue;
      if (isExact) {
        if (exact) list.add('=' + tok);
      } else list.add(tok);
    }
    if (!list.size) list.add(primary.replace(/[^A-Z0-9]/g, ''));
    const la = Number(lat);
    const lo = -Number(lon);
    if (!Number.isFinite(la) || !Number.isFinite(lo)) continue;
    lines.push(`${name.replace(/\|/g, ' ')}|${cont}|${la.toFixed(1)}|${lo.toFixed(1)}|${[...list].join(',')}`);
  }
  return lines;
}

function header(source) {
  return `/**
 * Prefix table: which country (DXCC entity) a callsign belongs to, and a point for the direction and
 * the distance. Made by tools/update_prefixes.mjs from ${source}. Do not edit by hand: run the tool again.
 *
 * Format of every line:   name|continent|latitude|longitude|prefixes
 *   latitude: degrees, north positive. longitude: degrees, east positive.
 *   prefixes: separated by commas. \`=DL1ABC\` is one exact callsign. The longest match wins.
 */
export const PREFIX_TABLE_VERSION = '${new Date().toISOString().slice(0, 10)} (${source})';

export const PREFIX_TABLE = \`
`;
}

function main(argv) {
  const args = argv.slice(2);
  const file = args.find((a) => !a.startsWith('--'));
  if (!file) {
    console.error('Usage: node tools/update_prefixes.mjs cty.dat [--exact] [--out file.ts]');
    process.exit(2);
  }
  const oi = args.indexOf('--out');
  const here = dirname(fileURLToPath(import.meta.url));
  const out = oi >= 0 ? args[oi + 1] : resolve(here, '../ui/src/lib/dxcc-data.ts');
  const lines = convertCty(readFileSync(file, 'utf8'), { exact: args.includes('--exact') });
  if (lines.length < 200) {
    console.error(`Only ${lines.length} entities found. Is this a cty.dat? Nothing written.`);
    process.exit(1);
  }
  const src = file.split(/[\\/]/).pop();
  writeFileSync(out, header(src) + lines.join('\n') + '\n`.trim();\n');
  console.log(`${lines.length} entities written to ${out}`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main(process.argv);
