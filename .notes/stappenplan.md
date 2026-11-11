# CWLD stappenplan en roadmap (levend document, stand 10 november 2026, na 0.13.1)

Regel: eerst modeladvies, dan pas beginnen na GO. Eén volledige zip per levering. Kleine dingen worden gebundeld in de volgende ronde. Een release start vanzelf bij een push naar `main` met een hoger versienummer in `src-tauri/tauri.conf.json` (geen tag nodig).

Ontwerpen: claude/ontwerp_architectuur.md (basis), claude/ontwerp_contest_profielen.md (C-1), claude/ontwerp_contest_oefenen_website.md (C-2, C-3, oefenen, website).

## Volgorde (besluit Kristof, 8 november)
1. C-2 contest mode en logboek (gebouwd, wacht op tests)
2. C-3 stationsprofielen, WWFF/ONFF-activaties, exportvarianten (gebouwd, 0.8.0, wacht op tests)
3. O oefenen (simulator) (gebouwd, 0.9.0, wacht op tests)
4. G website met laatste versie (gebouwd, wacht op publicatie)
5. F (0.11.0), P (0.12.0), Q (0.13.0) en herstelronde 0.13.1 gebouwd. Daarna: codesignatuur (onderzoek gedaan, wacht op GO), D Frans, E andere radio's
6. B benchmark tegen MRP40 helemaal op het einde

## Gedaan
- [x] Tot en met 0.4.4: ruisgate en zekerheidsmeter, auto-verbinden, nette afsluiting en installer-opruiming, Q-codepaneel, automatische release, bandscoop, tabjes.
- [x] MRP40-bevindingen vastgelegd (claude/mrp40_bevindingen.md), inclusief filtervergelijking.
- [x] Ronde A (0.5.0): instellingen in rubrieken, standaardrapport 5NN/599, venster maximaal, status volgt radio, pile-up F1 tot F4, Q-paneel Ctrl+Q, afkortingen met tooltip en eigen woorden, tabjes, piekmeter en gain, filterlijnen, Smal/Breed, Zwak/Midden/Sterk, snelheidsgrenzen en vaste snelheid, Sterkste en AFC, {NR} en contestschakelaar.
- [x] Ronde A2 (0.5.1): installer wacht tot cwld.exe weg is en toont welk proces hangt; simulator zichtbaar als "Simulator, geen radio".
- [x] Ronde C-1 (0.6.0): wedstrijdprofielen (9 meegeleverd plus leeg sjabloon), eigen profielen, eigen variabelen, keuzelijst en chips, verkorte cijfers, export en import als reservekopie, .gitattributes.
- [x] Ronde C-2 (0.7.0): logboek in SQLite, contest-invoerregel, N1MM-toetsen F1 tot F8, Enter als ESM, Run en S&P, dupewaarschuwing, rate, testmodus met apart testlog, klein logpaneel, ADIF-export met filters, bandscoop 165 pixels.
- [x] Ronde C-3 (0.8.0): stationsprofielen Thuis/Portable, activaties WWFF/ONFF/POTA/SOTA, exportvarianten, controlevenster.
- [x] Ronde O (0.9.0): Oefenen als simulator met oefenband, oefeningen, score, testlog, link naar de morse-leer-app.
- [x] Ronde W (0.10.0): begeleide eerste start.

## Te testen door Kristof
- [x] Bevonden 10 november: zijtoon komt op de USB-audio na MENU, SET, Connectors, USB AF/IF Output, "AF Beep/Speech... Output" ON. AF op 0 maakt ook die zijtoon stil; AF 20 procent met squelch dicht werkt (band stil, paddle herkend). QRZ-link werkt bij klikken, niet bij typen (hersteld in 0.13.1). Bandscoop viel uit, terug na herstart van de radio (geen CWLD-fout vastgesteld).
- [ ] 0.13.1: A40 (squelch en break-in, regel "Radio meldt"), A41 (paddle oefenen in eigen tabblad), A44 (QRZ bij typen), A43 (QRZ algemeen, welke velden geeft een gratis account).
- [ ] A42 (decoder bij SSB) en A39 (Log4OM).
- [ ] A34 (begeleide eerste start), A33 (oefenen), A31 en A32 (profielen, activatie, export).
- [ ] A26 tot A30 (contest, export naar Log4OM: STX, SRX, CONTEST_ID).
- [ ] A17 tot A25, ouder: auto-verbinden en update op Windows, A15, A1 tot A14, A16, Q10.

## Open kleine items
- [ ] UBA-sectielijst, CONTEST_ID per wedstrijd controleren, berichten F1 tot F8 per profiel, Winter Contest extra veld, Cabrillo, UBA DX Contest-datums.
- [ ] Referentiebestanden van Kristof (PoLo-export en Log4OM-export).
- [ ] Bandscoop: als het nog eens wegvalt, noteren wat er vlak voor gebeurde (OEFENEN verlaten, afsluiten, opnieuw verbinden).

## F: koppelingen (0.11.0)
- [x] Log4OM via ADIF-record per UDP-bericht (127.0.0.1:2333), N1MM-formaat contactinfo voor luisteraars.
- [ ] Test A39 in Log4OM [TEST].

## P: paddle oefenen en decoderpoort (0.12.0, hersteld in 0.13.1)
- [x] Paddle-oefenen met score, tips, statistiek per teken.
- [x] Decoder pauzeert bij USB, LSB, AM, FM, RTTY en bij spraak.
- [x] 0.13.1: radio stil via squelch dicht (CI-V 14 03, [TEST] op MK II) en break-in uit; AF-volume niet meer aangeraakt; regel "Radio meldt" met volume, squelch, break-in; waarschuwing bij AF 0; hint en handleiding met het menu AF Beep/Speech Output; elke oefening in een eigen tabblad "Oefenen".
- [x] 0.13.2: zijtoon zichtbaar in de audio maar niet herkend (ON3VZ). Oorzaak: zijtoon met boventonen gold voor de spraakcontrole als spraak. Spraakcontrole uit tijdens paddle oefenen; nieuwe sleuteldetector op de omhullende (exact in tests, ruis alleen geeft 0 key-downs); score leest de tekst uit de sleuteltijden. Metingen in claude/meetresultaten_decoder.md.
- [ ] A40 en A41 opnieuw met 0.13.2, A42 met echte SSB.
- Later: voortgangsgrafiek per week.

## Q: QRZ.com XML (0.13.0, hersteld in 0.13.1)
- [x] cwld-lookup met sessie, cache, terugval van /P, wachtwoord versleuteld in de OS-kluis.
- [x] Vult lege velden, of velden die CWLD zelf invulde voor een vorig station; wat de operator typte blijft.
- [x] 0.13.1: getypte roepletters worden ook opgezocht (was alleen bij klikken); grijze regel alleen voor de roepletters in het paneel.
- [ ] A43 en A44.
- Later: HamQTH als gratis tweede bron, land en locator in het logboek.

## K: Koch-methode (0.14.0)
- [x] Tekens kiezen in Koch-volgorde (schuif en losse tekens), bewaard in settings `practice.koch`.
- [x] Paddle-oefeningen alleen met gekozen tekens; woorden, calls en zinnen alleen als ze volledig passen.
- [x] Luisteroefening via de pc met Farnsworth, typen, fouten per teken, 90 procent-voorstel (nooit automatisch).
- [ ] Test A45. Later: kopieerreeksen met echte roepletters, voortgangsgrafiek.

## D2: decoder voor slordige morse (0.15.0)
- [x] Paddle oefenen: sleuteldetector zoekt de zijtoon zelf (300 tot 1200 Hz), los van de decodertoon.
- [x] Testbank: 14 slordige vuisten (`cwld_bench --fists`), `seekbench`, generator met aparte pauzes.
- [x] Woordgrenzen per zender met woordkennis (`spacing.rs`), stijl van de zender leren (weging, spatiëring), herstel van gebroken en samengesmolten tekens, begin van een uitzending bij toon zoeken.
- [x] Metingen in claude/meetresultaten_decoder.md. Zeven opnames van 10 november als vergelijking (tekst nog niet bevestigd, opnames staan niet in de repo).
- [x] 0.15.1: één verloren element terugzetten (roepletters die net gehoord zijn, gewone woorden), Engelse woordenlijst, minder foute splitsingen. Opname 7026,8 kHz van 11 november als extra vergelijking.
- [x] 0.16.0: zwakke elementen in snelle fading tellen mee (boven de ruis, hoogstens 12 dB onder het niveau, op de toon, niveau houdt). Gemengd resultaat, zie meetresultaten.
- [x] 0.16.0: wens Kristof genoteerd en gebouwd: Koch ook voor zenden. Knoppen "Sein groepen" en "Sein woorden" in de Koch-kaart; paddle-oefening start zelf de audio.
- [x] 0.16.1: schakelaar "Fading (QSB)" op het hoofdscherm (kaart Audio): Doorlezen of Streng, met uitleg en nadeel in beeld. Test A49.
- [ ] A46 en A47. Juiste tekst van de zeven opnames van Kristof.
- [ ] Open: -3 dB op 36 WPM werd slechter (66 naar 75), `THE SCREEN` in de opname van 7026,8 kHz, zender die van tempo wisselt binnen een uitzending, cel 615 Hz bij +3 dB in seekbench, draaggolf 120 Hz (4,7 naar 6,1).

## Punt 4: decoderkwaliteit (keuze Kristof, 11 november)
- [x] 4a (0.17.0): zender die van spatiëring wisselt. Beter op de testbank, geen verschil op de acht opnames.
- [ ] 4b: roepletters uit eigen logboek en QSO-paneel meegeven (Sonnet). Daarna lijst van bekende roepletters, licentie eerst nakijken.
- [x] 4c (0.18.3): klein neuraal netwerk geprobeerd (6001 gewichten, numpy op CPU, sleutelstand per frame). Geen winst op de testbank (10,7 tegen 10,8 procent), dus niet ingeschakeld. Code en gereedschap blijven staan. Vervolg alleen met echte opnames met juiste tekst, of een netwerk dat tekens leest (veel groter).
- [x] 0.18.3: paddle-oefening telt letters los van spaties (melding Kristof: 20 procent bij één foute letter door lange pauzes).
- [x] 4d-1 (0.18.0): clusterspots op de bandscoop. Telnet-client met inloggen, skimmer aan of uit, uitval naar de volgende node, lijst met acht clusters (ON0DXK eerst), label per station met kleur (nieuw, andere band, al gewerkt of dupe), klik stemt af, schakelaar tonen of verbergen, Test-knop, commandovenster dat alleen leest. Test A50 tot A52. Nog niet getest op de echte nodes.
- [x] 0.18.1: labels rustiger (grijze vakjes met gekleurd bolletje, gewerkt vervaagt) en spots binnen 700 Hz van de VFO als hint voor de decoder. Hintbench: roepletter goed 60 naar 85 procent bij 3 op 100 elementen weg, foute hints zonder effect. Test A53.
- [x] 0.18.2: bandscoop opnieuw getekend na opmerking Kristof (blokkerig, alles bovenaan gepropt): eigen strook voor labels, vloeiend spectrum met automatisch niveau, schaal in eigen strook, vloeiende waterval met nieuw palet. Test A54.
- [x] 0.18.4: opnames van 11 november tonen een gat van 35 tot 45 dB tussen 450 en 850 Hz (op 10 november niet), vermoedelijk de notch van de radio. Waarschuwing in de kaart Audio, toonzoeker volgt geen filterrand meer. Test A55. Open: met het gat kiest de toonzoeker soms een ander signaal (1167 Hz in de opname van 7012,1 kHz); eerst afwachten of de notch de oorzaak was.
- [ ] 4b zoals eerst bedoeld (logboek als bron) vervalt: weinig toegevoegde waarde volgens Kristof, vervangen door de spot-hints.
- [ ] 4d-2: eigen decoders per signaal in de doorlaat (Opus), later en optioneel. Eerst CPU meten. Schakelaar met belastingsbeveiliging: hoofdkanaal heeft voorrang, extra kanalen vallen weg bij te hoge belasting, belastingsindicator, standaard uit. Beperkt door het radiofilter. Labels uit 4d-1 krijgen dan een tweede bron.
- [ ] Later, optioneel: een WebSDR of KiwiSDR als tweede ontvanger (alleen online). KiwiSDR heeft een open koppeling met ongeveer 12 kHz bandbreedte en tijdslimieten; WebSDR heeft voor zover bekend geen open koppeling. Eerst voorwaarden nakijken.

## Codesignatuur (onderzoek 10 november, wacht op GO)
- Advies: SignPath Foundation (gratis voor open source, uitgever "SignPath Foundation", code signing policy-pagina en privacyverklaring nodig, MFA verplicht, elke release goedkeuren; vraag of één onderhouder mag). Reserve: Certum Open Source (25 of 49 euro per jaar, alleen particulieren, cloudvariant mogelijk niet onbeheerd in CI, was uitverkocht). Azure Artifact Signing: particulieren alleen VS en Canada.
- Geen enkele optie haalt SmartScreen meteen weg; reputatie bouwt op.

## G: website en verpakking
- [x] Site in `site/`, downloadknop, gids, workflow pages.yml.
- [ ] Eenmalig: GitHub Settings, Pages, Source = GitHub Actions.

## D: Frans (fr.json, CW-termen met Kristof controleren)

## E: radio-onderzoek (Opus)
IC-705 eerst, daarna IC-7610, IC-9700, IC-7851, Yaesu FT-710, Xiegu G90.

## Later
- Groot scherm (richting B), meerdere decoders tegelijk, roepletterlijst voor controle.

## B: benchmark tegen MRP40 (Opus), helemaal op het einde
- [ ] B1 tot B3 en drempels Zwak/Midden/Sterk meten.

## Open vragen aan Kristof
1. Wat stond er naast "Verbonden" en welk Type radio was ingesteld toen de status niet klopte?
2. Grid JO20EE thuis en 5 W portable als standaard?
3. Ouder: SNST in de DL1NK-opname en R330BF?
