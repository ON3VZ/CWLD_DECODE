export interface Message {
  id: number;
  /** rx: decoded from the other station. tx: typed and sent from the app. paddle: keyed on the radio, decoded. practice: simulated. */
  from: 'rx' | 'tx' | 'paddle' | 'practice';
  call?: string;
  text: string;
  /** Decoder confidence per character of `text` (0..1). Missing: all sure. */
  conf?: number[];
  time: string;
}

export interface Signal {
  id: number;
  offsetHz: number;
  wpm: number;
  snr: number;
  call: string;
}

/** Demo data for the first screen. Replaced by real decoder output in a later phase. */
export const demoMessages: Message[] = [
  { id: 1, from: 'rx', call: 'DL1ABC', text: 'CQ CQ CQ DE DL1ABC DL1ABC K', time: '14:02' },
  { id: 2, from: 'paddle', text: 'DL1ABC DE ON3VZ ON3VZ K', time: '14:02' },
  { id: 3, from: 'rx', call: 'DL1ABC', text: 'ON3VZ DE DL1ABC TNX FER CALL UR RST 579 579 NAME HANS QTH BERLIN HW? K', time: '14:03' },
  { id: 4, from: 'tx', text: 'DL1ABC DE ON3VZ TNX HANS UR RST 599 599 K', time: '14:03' },
  { id: 5, from: 'paddle', text: 'R R TNX HANS BT', time: '14:04' },
];

export const demoSignals: Signal[] = [
  { id: 1, offsetHz: -140, wpm: 18, snr: 9, call: 'DL1ABC' },
  { id: 2, offsetHz: 60, wpm: 24, snr: 14, call: 'F4XYZ' },
  { id: 3, offsetHz: 210, wpm: 15, snr: 4, call: '' },
];

export const defaultChips = [
  { id: 'cq', key: 'chip.cq', template: 'CQ CQ CQ DE {MYCALL} {MYCALL} K' },
  { id: 'answer', key: 'chip.answer', template: '{CALL} DE {MYCALL} {MYCALL} K' },
  { id: 'report', key: 'chip.report', template: '{CALL} DE {MYCALL} UR RST {RST} {RST} NAME {MYNAME} QTH {MYQTH} HW? K' },
  { id: 'agn', key: 'chip.agn', template: 'AGN?' },
  { id: 'qrz', key: 'chip.qrz', template: 'QRZ? DE {MYCALL} K' },
  { id: 'bye', key: 'chip.bye', template: '{CALL} DE {MYCALL} TNX FER QSO 73 SK' },
];

/** Quick replies for a pile-up, on the keys F1 to F4. {RST} is the report in the QSO panel (5NN by default). */
export const pileupChips = [
  { id: 'pu1', key: 'chip.pu1', fkey: 'F1', template: '{MYCALL}' },
  { id: 'pu2', key: 'chip.pu2', fkey: 'F2', template: '{MYCALL} {MYCALL}' },
  { id: 'pu3', key: 'chip.pu3', fkey: 'F3', template: 'R {RST} TU' },
  { id: 'pu4', key: 'chip.pu4', fkey: 'F4', template: 'AGN?' },
];

/** Extra chips in contest mode. {NR} is the serial number. */
export const contestChips = [
  { id: 'exchange', key: 'chip.exchange', template: '{CALL} {RST} {NR}' },
];
