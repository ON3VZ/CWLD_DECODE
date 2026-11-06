/**
 * Q-codes, common CW abbreviations and prosigns, with meaning in English and Dutch.
 * Q-codes: ITU-R M.1172 / amateur use. A code followed by "?" is a question,
 * without it an answer or statement. The `ask` text is the question, `say` the answer.
 */
export type QKind = 'q' | 'abbr' | 'prosign';

export interface QEntry {
  /** What is typed in the send box. Prosigns are written as <AR>. */
  code: string;
  kind: QKind;
  /** The meaning as a statement. */
  en: string;
  nl: string;
  /** Meaning when sent as a question (code followed by ?), only for Q-codes. */
  askEn?: string;
  askNl?: string;
  /** Prosigns only: the Morse pattern. */
  pattern?: string;
  /** Prosigns that the keyer cannot send are shown but not inserted. */
  insertable?: boolean;
}

export const Q_ENTRIES: QEntry[] = [
  { code: 'QRA', kind: 'q', en: 'The name of my station is ...', nl: 'De naam van mijn station is ...', askEn: 'What is the name of your station?', askNl: 'Wat is de naam van uw station?' },
  { code: 'QRG', kind: 'q', en: 'Your exact frequency is ...', nl: 'Uw exacte frequentie is ...', askEn: 'Will you tell me my exact frequency?', askNl: 'Wilt u mij mijn exacte frequentie opgeven?' },
  { code: 'QRH', kind: 'q', en: 'Your frequency varies.', nl: 'Uw frequentie varieert.', askEn: 'Does my frequency vary?', askNl: 'Varieert mijn frequentie?' },
  { code: 'QRI', kind: 'q', en: 'The tone of your transmission is ... (1 good, 2 variable, 3 bad).', nl: 'De toon van uw seinen is ... (1 goed, 2 wisselend, 3 slecht).', askEn: 'What is the tone of my transmission?', askNl: 'Hoe is de toon van mijn seinen?' },
  { code: 'QRK', kind: 'q', en: 'The readability of your signals is ... (1 to 5).', nl: 'De leesbaarheid van uw signalen is ... (1 tot 5).', askEn: 'What is the readability of my signals?', askNl: 'Hoe is de leesbaarheid van mijn signalen?' },
  { code: 'QRL', kind: 'q', en: 'I am busy, please do not interfere.', nl: 'Ik ben bezig, stoor niet.', askEn: 'Is this frequency in use?', askNl: 'Is de frequentie bezet?' },
  { code: 'QRM', kind: 'q', en: 'I am troubled by interference from other stations.', nl: 'Ik word gestoord door andere stations.', askEn: 'Are you troubled by interference?', askNl: 'Wordt u gestoord door andere stations?' },
  { code: 'QRN', kind: 'q', en: 'I am troubled by static (atmospheric noise).', nl: 'Ik word gestoord door atmosferische storing.', askEn: 'Are you troubled by static?', askNl: 'Wordt u gestoord door atmosferische storing?' },
  { code: 'QRO', kind: 'q', en: 'Increase power.', nl: 'Verhoog het vermogen.', askEn: 'Shall I increase power?', askNl: 'Moet ik het vermogen verhogen?' },
  { code: 'QRP', kind: 'q', en: 'Decrease power (low power).', nl: 'Verlaag het vermogen (laag vermogen).', askEn: 'Shall I decrease power?', askNl: 'Moet ik het vermogen verlagen?' },
  { code: 'QRQ', kind: 'q', en: 'Send faster.', nl: 'Sein sneller.', askEn: 'Shall I send faster?', askNl: 'Moet ik sneller seinen?' },
  { code: 'QRS', kind: 'q', en: 'Send slower.', nl: 'Sein langzamer.', askEn: 'Shall I send more slowly?', askNl: 'Moet ik langzamer seinen?' },
  { code: 'QRT', kind: 'q', en: 'I am stopping transmission (closing down).', nl: 'Ik stop met seinen (sluit af).', askEn: 'Shall I stop sending?', askNl: 'Moet ik stoppen met seinen?' },
  { code: 'QRU', kind: 'q', en: 'I have nothing for you.', nl: 'Ik heb niets voor u.', askEn: 'Have you anything for me?', askNl: 'Hebt u iets voor mij?' },
  { code: 'QRV', kind: 'q', en: 'I am ready.', nl: 'Ik ben gereed.', askEn: 'Are you ready?', askNl: 'Bent u gereed?' },
  { code: 'QRX', kind: 'q', en: 'I will call you again later (stand by).', nl: 'Ik roep u later weer op (wacht even).', askEn: 'When will you call me again?', askNl: 'Wanneer roept u mij weer op?' },
  { code: 'QRZ', kind: 'q', en: 'You are being called by ...', nl: 'U wordt aangeroepen door ...', askEn: 'Who is calling me?', askNl: 'Wie roept mij op?' },
  { code: 'QSA', kind: 'q', en: 'The strength of your signals is ... (1 to 5).', nl: 'De sterkte van uw signalen is ... (1 tot 5).', askEn: 'What is the strength of my signals?', askNl: 'Hoe sterk zijn mijn signalen?' },
  { code: 'QSB', kind: 'q', en: 'Your signals are fading.', nl: 'Uw signalen fadingen (gaan op en neer).', askEn: 'Are my signals fading?', askNl: 'Faden mijn signalen?' },
  { code: 'QSK', kind: 'q', en: 'I can hear you between my signals, you may break in (full break-in).', nl: 'Ik hoor u tussen mijn signalen, u mag tussenin komen (volledige break-in).', askEn: 'Can you hear me between your signals?', askNl: 'Kunt u mij horen tussen uw signalen?' },
  { code: 'QSL', kind: 'q', en: 'I acknowledge receipt (also: I will send a QSL card).', nl: 'Ik bevestig ontvangst (ook: ik stuur een QSL-kaart).', askEn: 'Can you acknowledge receipt?', askNl: 'Kunt u de ontvangst bevestigen?' },
  { code: 'QSM', kind: 'q', en: 'I will repeat my last message.', nl: 'Ik herhaal mijn laatste bericht.', askEn: 'Shall I repeat the last message?', askNl: 'Moet ik het laatste bericht herhalen?' },
  { code: 'QSO', kind: 'q', en: 'I can communicate with ... directly.', nl: 'Ik kan rechtstreeks met ... werken.', askEn: 'Can you communicate with ... directly?', askNl: 'Kunt u rechtstreeks met ... werken?' },
  { code: 'QSP', kind: 'q', en: 'I will relay to ...', nl: 'Ik geef door aan ...', askEn: 'Will you relay to ...?', askNl: 'Wilt u doorgeven aan ...?' },
  { code: 'QSX', kind: 'q', en: 'I am listening to ... on ...', nl: 'Ik luister naar ... op ...', askEn: 'Will you listen to ... on ...?', askNl: 'Wilt u luisteren naar ... op ...?' },
  { code: 'QSY', kind: 'q', en: 'I change to another frequency.', nl: 'Ik ga naar een andere frequentie.', askEn: 'Shall I change to another frequency?', askNl: 'Moet ik naar een andere frequentie gaan?' },
  { code: 'QTC', kind: 'q', en: 'I have ... messages for you.', nl: 'Ik heb ... berichten voor u.', askEn: 'How many messages have you to send?', askNl: 'Hoeveel berichten hebt u te seinen?' },
  { code: 'QTH', kind: 'q', en: 'My location is ...', nl: 'Mijn locatie is ...', askEn: 'What is your location?', askNl: 'Wat is uw locatie?' },
  { code: 'QTR', kind: 'q', en: 'The exact time is ...', nl: 'De exacte tijd is ...', askEn: 'What is the exact time?', askNl: 'Hoe laat is het precies?' },

  { code: 'CQ', kind: 'abbr', en: 'General call to all stations.', nl: 'Algemene oproep aan alle stations.' },
  { code: 'DE', kind: 'abbr', en: 'From (this is).', nl: 'Van (dit is).' },
  { code: 'K', kind: 'abbr', en: 'Go ahead, anyone may reply.', nl: 'Kom maar, iedereen mag antwoorden.' },
  { code: 'R', kind: 'abbr', en: 'Received, all copied.', nl: 'Ontvangen, alles goed.' },
  { code: 'RST', kind: 'abbr', en: 'Readability, strength, tone report.', nl: 'Leesbaarheid, sterkte, toon (rapport).' },
  { code: 'TNX', kind: 'abbr', en: 'Thanks (also TKS).', nl: 'Bedankt (ook TKS).' },
  { code: 'TU', kind: 'abbr', en: 'Thank you.', nl: 'Dank u.' },
  { code: 'FB', kind: 'abbr', en: 'Fine business, very good.', nl: 'Fijne zaak, heel goed.' },
  { code: 'UFB', kind: 'abbr', en: 'Ultra fine business, excellent.', nl: 'Uitstekend.' },
  { code: 'HW', kind: 'abbr', en: 'How (HW? = how do you copy me?).', nl: 'Hoe (HW? = hoe ontvangt u mij?).' },
  { code: 'AGN', kind: 'abbr', en: 'Again, please repeat.', nl: 'Opnieuw, herhaal alstublieft.' },
  { code: 'PSE', kind: 'abbr', en: 'Please.', nl: 'Alstublieft.' },
  { code: 'RPT', kind: 'abbr', en: 'Repeat.', nl: 'Herhaal.' },
  { code: 'CFM', kind: 'abbr', en: 'Confirm.', nl: 'Bevestig.' },
  { code: 'ABT', kind: 'abbr', en: 'About.', nl: 'Over, omtrent.' },
  { code: 'UR', kind: 'abbr', en: 'Your (or you are).', nl: 'Uw (of u bent).' },
  { code: 'HR', kind: 'abbr', en: 'Here.', nl: 'Hier.' },
  { code: 'ES', kind: 'abbr', en: 'And.', nl: 'En.' },
  { code: 'FER', kind: 'abbr', en: 'For.', nl: 'Voor.' },
  { code: 'WID', kind: 'abbr', en: 'With.', nl: 'Met.' },
  { code: 'WX', kind: 'abbr', en: 'Weather.', nl: 'Weer.' },
  { code: 'TEMP', kind: 'abbr', en: 'Temperature.', nl: 'Temperatuur.' },
  { code: 'ANT', kind: 'abbr', en: 'Antenna.', nl: 'Antenne.' },
  { code: 'RIG', kind: 'abbr', en: 'Transceiver, radio.', nl: 'Zendontvanger, radio.' },
  { code: 'PWR', kind: 'abbr', en: 'Power.', nl: 'Vermogen.' },
  { code: 'OP', kind: 'abbr', en: 'Operator.', nl: 'Operator.' },
  { code: 'NAME', kind: 'abbr', en: 'Name.', nl: 'Naam.' },
  { code: 'OM', kind: 'abbr', en: 'Old man, friendly for a male ham.', nl: 'Oude man, vriendelijk voor een zendamateur.' },
  { code: 'YL', kind: 'abbr', en: 'Young lady.', nl: 'Dame (zendamateur).' },
  { code: 'XYL', kind: 'abbr', en: 'Wife.', nl: 'Echtgenote.' },
  { code: 'DX', kind: 'abbr', en: 'Long distance, far away station.', nl: 'Verre afstand, ver weg station.' },
  { code: 'GM', kind: 'abbr', en: 'Good morning.', nl: 'Goedemorgen.' },
  { code: 'GA', kind: 'abbr', en: 'Good afternoon (or go ahead).', nl: 'Goedemiddag (of ga door).' },
  { code: 'GE', kind: 'abbr', en: 'Good evening.', nl: 'Goedenavond.' },
  { code: 'GN', kind: 'abbr', en: 'Good night.', nl: 'Goedenacht.' },
  { code: 'GL', kind: 'abbr', en: 'Good luck.', nl: 'Veel succes.' },
  { code: 'CUL', kind: 'abbr', en: 'See you later.', nl: 'Tot later.' },
  { code: 'HPE', kind: 'abbr', en: 'Hope.', nl: 'Hopen.' },
  { code: 'HI', kind: 'abbr', en: 'Laughter.', nl: 'Gelach.' },
  { code: 'SRI', kind: 'abbr', en: 'Sorry.', nl: 'Sorry.' },
  { code: 'VY', kind: 'abbr', en: 'Very.', nl: 'Zeer.' },
  { code: 'NR', kind: 'abbr', en: 'Number.', nl: 'Nummer.' },
  { code: 'NW', kind: 'abbr', en: 'Now.', nl: 'Nu.' },
  { code: 'SIG', kind: 'abbr', en: 'Signal.', nl: 'Signaal.' },
  { code: 'TEST', kind: 'abbr', en: 'Contest call (CQ TEST).', nl: 'Wedstrijdoproep (CQ TEST).' },
  { code: 'B4', kind: 'abbr', en: 'Before, heard before.', nl: 'Eerder, al eerder gehoord.' },
  { code: 'NIL', kind: 'abbr', en: 'Nothing.', nl: 'Niets.' },
  { code: '73', kind: 'abbr', en: 'Best regards.', nl: 'Beste groeten.' },
  { code: '72', kind: 'abbr', en: 'Best regards (QRP stations).', nl: 'Beste groeten (QRP-stations).' },
  { code: '88', kind: 'abbr', en: 'Love and kisses.', nl: 'Liefs en kussen.' },

  { code: '<AR>', kind: 'prosign', en: 'End of message.', nl: 'Einde bericht.', pattern: '.-.-.', insertable: true },
  { code: '<SK>', kind: 'prosign', en: 'End of contact (silent key).', nl: 'Einde verbinding.', pattern: '...-.-', insertable: true },
  { code: '<BT>', kind: 'prosign', en: 'Break, new paragraph (also =).', nl: 'Pauze, nieuwe alinea (ook =).', pattern: '-...-', insertable: true },
  { code: '<KN>', kind: 'prosign', en: 'Go ahead, only the station called.', nl: 'Kom maar, alleen het opgeroepen station.', pattern: '-.--.', insertable: true },
  { code: '<AS>', kind: 'prosign', en: 'Wait.', nl: 'Wacht even.', pattern: '.-...', insertable: true },
  { code: '<BK>', kind: 'prosign', en: 'Break, back to you.', nl: 'Break, terug naar u.', pattern: '-...-.-', insertable: true },
  { code: '<SN>', kind: 'prosign', en: 'Understood (also VE).', nl: 'Begrepen (ook VE).', pattern: '...-.', insertable: true },
  { code: '<HH>', kind: 'prosign', en: 'Error, I will correct.', nl: 'Fout, ik verbeter.', pattern: '........', insertable: true },
  { code: '<CL>', kind: 'prosign', en: 'Closing down, going off the air.', nl: 'Sluit af, uit de lucht.', pattern: '-.-..-..', insertable: false },
];

export function entryMeaning(e: QEntry, lang: string, asQuestion = false): string {
  const nl = lang === 'nl';
  if (asQuestion && e.askEn) return (nl ? e.askNl : e.askEn) ?? e.en;
  return nl ? e.nl : e.en;
}

/** Filter by a search text: matches the code (start first) or any word of the meaning. */
export function searchEntries(list: QEntry[], query: string, lang: string): QEntry[] {
  const q = query.trim().toUpperCase().replace(/[<>?]/g, '');
  if (!q) return list;
  const starts: QEntry[] = [];
  const other: QEntry[] = [];
  for (const e of list) {
    const code = e.code.replace(/[<>]/g, '');
    if (code.startsWith(q)) starts.push(e);
    else if (code.includes(q) || entryMeaning(e, lang).toUpperCase().includes(q) || (e.askEn ?? '').toUpperCase().includes(q) || (e.askNl ?? '').toUpperCase().includes(q)) other.push(e);
  }
  return [...starts, ...other];
}

/** Look up a plain word (for tooltips in the conversation). */
export function lookup(word: string): QEntry | undefined {
  const w = word.toUpperCase().replace(/[?.,]+$/g, '');
  return Q_ENTRIES.find((e) => e.code === w || e.code === `<${w}>`);
}

/** Words of a text that are known Q-codes, abbreviations, prosigns or own words, once each, in order of appearance. */
export function usedIn(text: string, own: { code: string; meaning: string }[] = []): { code: string; entry?: QEntry; own?: string }[] {
  const seen = new Set<string>();
  const out: { code: string; entry?: QEntry; own?: string }[] = [];
  for (const raw of text.toUpperCase().split(/\s+/)) {
    const w = raw.replace(/[?.,;:!]+$/g, '');
    if (!w || seen.has(w)) continue;
    const o = own.find((x) => x.code.toUpperCase() === w);
    const e = lookup(w);
    if (o) {
      seen.add(w);
      out.push({ code: w, own: o.meaning });
    } else if (e && (!/^\d+$/.test(w) || ['72', '73', '88'].includes(w))) {
      seen.add(w);
      out.push({ code: w, entry: e });
    }
  }
  return out;
}
