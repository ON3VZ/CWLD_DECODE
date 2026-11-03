const CALL = /^(?:[A-Z]{1,2}|[0-9][A-Z])[0-9][A-Z]{1,4}(?:\/[A-Z0-9]{1,4})?$/;

export function looksLikeCall(word: string): boolean {
  return CALL.test(word.toUpperCase());
}

export interface Part {
  text: string;
  call: boolean;
}

/** Splits a decoded text into plain words and callsign-like words, so callsigns can be clicked. */
export function splitCalls(text: string): Part[] {
  return text
    .split(/(\s+)/)
    .filter((s) => s.length > 0)
    .map((s) => ({ text: s, call: !/^\s+$/.test(s) && looksLikeCall(s) }));
}
