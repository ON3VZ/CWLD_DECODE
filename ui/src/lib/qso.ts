import { looksLikeCall } from './callsign';

export interface Heard {
  /** The station that is talking (the sender of the text), if one can be told. */
  call?: string;
  name?: string;
  qth?: string;
}

const clean = (w: string) => w.replace(/[^A-Z0-9/]/g, '');

/**
 * Picks facts out of decoded text from the other station.
 * The sender is the callsign after `DE`. Without `DE` the first callsign that is not one of ours.
 * Name and QTH are the word after `NAME` or `QTH`. Nothing is guessed when the keyword is missing.
 */
export function heardIn(text: string, myCalls: string[]): Heard {
  const mine = new Set(myCalls.map((c) => c.toUpperCase()));
  const words = text.toUpperCase().split(/\s+/).map(clean).filter(Boolean);
  const out: Heard = {};

  for (let i = 0; i < words.length - 1; i++) {
    if (words[i] === 'DE' && looksLikeCall(words[i + 1]) && !mine.has(words[i + 1])) {
      out.call = words[i + 1];
      break;
    }
  }
  if (!out.call) out.call = words.find((w) => looksLikeCall(w) && !mine.has(w));

  const after = (key: string) => {
    const i = words.indexOf(key);
    const w = i >= 0 ? words[i + 1] : undefined;
    return w && !['NAME', 'QTH', 'RST', 'HW', 'K', 'KN', 'AR', 'SK'].includes(w) ? w : undefined;
  };
  out.name = after('NAME') ?? after('OP');
  out.qth = after('QTH');
  return out;
}
