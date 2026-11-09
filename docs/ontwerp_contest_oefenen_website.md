# CWLD ontwerp: contest mode, logboek, oefenen en website (voorstel ter goedkeuring)

Stand 8 november 2026. Ronde C-2 (punten 2 tot 7 en 9) is gebouwd in versie 0.7.0. De rest is nog een voorstel. Kristof keurt het goed of past het aan, daarna volgt de bouw in drie rondes. De benchmark tegen MRP40 schuift naar het einde (besluit Kristof, 8 november).

## 1. Hoe een ervaren contester werkt (uitgangspunten)

- **Het toetsenbord doet alles.** Tijdens een wedstrijd gaat de hand niet naar de muis. Elke handeling heeft een toets.
- **Spiergeheugen.** Wie contest doet, kent de toetsindeling van N1MM Logger+ (de meest gebruikte contestlogger). CWLD neemt in contest mode dezelfde indeling over. Dat is een idee en een werkwijze, geen code.
- **Run en S&P.** Je roept zelf CQ (Run) of je zoekt stations en roept ze aan (Search and Pounce, S&P). De toetsen gedragen zich anders in beide gevallen.
- **Enter doet het volgende logische ding** (in N1MM heet dat ESM, Enter Sends Message). Je hoeft niet na te denken welke F-toets.
- **Loggen gebeurt met TU.** Wie TU zendt, logt meteen. Pas dan is het volgnummer definitief.
- **Dupes zie je vóór je antwoordt**, niet achteraf.
- **Nooit wachten op de software.** Zenden en loggen mogen nooit vastlopen op een trage schijf of een radio die niet antwoordt.

## 2. Contestscherm

Onder de chips komt in contest mode een **invoerregel**:

| Roepletters | Rapport ontvangen | Uitwisseling ontvangen (volgens profiel) | Jouw nummer | Loggen |
|---|---|---|---|---|
| ON4ABC | 5NN | 123 ANR | 042 | knop |

- Het rapport ontvangen staat standaard op 5NN (of 599 volgens je instelling). Dat verandert bijna nooit.
- De kolom Uitwisseling ontvangen volgt het profiel: bij UBA nummer plus sectie (sectie alleen bij Belgische stations), bij CQ WW de zone, bij WPX en WAE het nummer.
- Daarboven één **inforegel**: DUPE in rood of "eerder gewerkt op 40 m" in grijs, band, aantal QSO's, rate (QSO's in de laatste 10 en 60 minuten), Run of S&P, en TEST in oranje als de testmodus aan staat.
- Rechts onder de audio-scope het **kleine logpaneel** (de laatste 5 QSO's), zoals Kristof eerder vroeg. Dubbelklik op een regel om hem te verbeteren.

## 3. Toetsen in contest mode (indeling zoals N1MM)

| Toets | Run | S&P |
|---|---|---|
| F1 | CQ (oproepzin uit het profiel + jouw roepletters) | jouw roepletters |
| F2 | uitwisseling | uitwisseling |
| F3 | TU + jouw roepletters, en loggen | TU, en loggen |
| F4 | jouw roepletters | jouw roepletters |
| F5 | roepletters van het andere station | idem |
| F6 | QSO B4 (al gewerkt) | idem |
| F7 | ? | idem |
| F8 | AGN? (nummer of sectie opnieuw: NR? of SEC?) | idem |
| Enter | zie hieronder | zie hieronder |
| Esc | stop zenden | stop zenden |
| Ctrl+W | invoerregel leegmaken | idem |
| Spatie | naar het volgende veld | idem |
| Ctrl+R | wisselen tussen Run en S&P | idem |

**Enter in Run:** lege roepletters zendt CQ. Roepletters ingevuld zendt roepletters plus uitwisseling. Uitwisseling ontvangen ingevuld zendt TU en logt. Ontbreekt de uitwisseling, dan zendt Enter AGN?.

**Enter in S&P:** roepletters ingevuld zendt jouw roepletters. Uitwisseling ontvangen ingevuld zendt jouw uitwisseling en logt.

De decoder-toetsen F6 (Sterkste) en F7 (AFC) worden in contest mode Ctrl+F6 en Ctrl+F7. Buiten contest mode blijft alles zoals nu, ook de pile-up-toetsen F1 tot F4.

De berichten achter F1 tot F8 krijgen standaardteksten en zijn per profiel aan te passen (later in de editor, eerst met vaste standaarden).

## 4. Logboek

- **Eén logboek voor de hele app**, ook voor gewone QSO's buiten een wedstrijd. In het QSO-paneel komt daarvoor een knop Loggen.
- De Rust-kern bewaart het logboek in de gebruikersmap in een SQLite-bestand `logbook.db`, zoals al in het architectuurontwerp (hoofdstuk 3.11) stond: betrouwbaar bij stroomuitval of een crash, en snel voor dupes. ADIF is het uitwisselformaat, niet de bron. Bij elke start maakt CWLD een reservekopie. De licentie van de SQLite-bibliotheek wordt bij de bouw nagekeken.
- Per QSO: datum en tijd (UTC), roepletters, band, frequentie, modus CW, rapport gezonden en ontvangen, nummer gezonden en ontvangen, extra veld gezonden en ontvangen, naam en QTH (bij gewone QSO's), profiel en wedstrijd, jouw roepletters, test ja of nee.
- **Volgnummer:** pas definitief bij loggen. Is de uitwisseling al gezonden maar nog niet gelogd en typ je een nieuwe roepletter, dan verschijnt een waarschuwing: "nummer 042 is gezonden maar niet gelogd".
- **Dupe:** zelfde roepletters, band en modus binnen dezelfde wedstrijd. Alleen een waarschuwing, je kunt toch loggen.
- **Een wedstrijd** begint als je contest mode aanzet met een profiel en kiest voor "Nieuwe wedstrijd" (nummer terug naar 001, dupes vanaf nu). Doorgaan houdt nummer en dupes.
- Verbeteren en verwijderen kan in het logpaneel. Een verwijderde QSO krijgt een markering in het bestand, zodat er niets verloren gaat.

## 5. Testmodus (het nummer testen zonder dat het telt)

- In de contestbalk een schakelaar **Test** (oranje). Aan: QSO's gaan naar een apart testlogboek, met een eigen teller vanaf 001. Het echte nummer blijft staan.
- Testlog-QSO's tellen niet voor dupes, komen niet in de export van het echte logboek en zijn met één knop te wissen.
- Oefenen (zie 8) gebruikt altijd de testmodus.

## 6. Gegevens van het ontvangen station

- **Klikken in het gesprek:** klik op een woord of getal en kies in een klein menu waar het heen moet: Roepletters, Rapport, Nummer, Extra veld (bijvoorbeeld Sectie), Naam, QTH. Het komt in de invoerregel of het QSO-paneel.
- **Voorstel van de decoder:** na je uitwisseling herkent CWLD in het antwoord een rapport (5NN, 599, 579), een nummer en een sectiecode of zone. Die verschijnen grijs als voorstel. Tab of Enter neemt ze over, of je overschrijft ze.
- **UBA-sectielijst** met aanvullen bij typen. De lijst komt uit het officiële UBA-reglement en wordt met Kristof gecontroleerd.

## 7. Export naar ADIF

- Knop Exporteren in het logboek, met van-tot (datum en tijd), band, profiel of wedstrijd, en test wel of niet.
- De ADIF-schrijver volgt hoofdstuk 3.11 D en E van het architectuurontwerp (ADI-formaat, alleen ASCII, UTC, controle vóór export). Velden volgens de ADIF-specificatie: CALL, QSO_DATE, TIME_ON, BAND, FREQ, MODE, RST_SENT, RST_RCVD, STX en SRX (nummers), STX_STRING en SRX_STRING (sectie of zone), CONTEST_ID, OPERATOR, STATION_CALLSIGN, NAME, QTH. De juiste CONTEST_ID-waarden per wedstrijd worden bij de bouw opgezocht in de officiële ADIF-lijst, niet geraden.
- Het bestand kan in Log4OM worden ingelezen (in Log4OM: menu Bestand, ADIF importeren). De rechtstreekse koppeling met Log4OM is ronde F.

## 8. Oefenen: wat het moet zijn

**Het probleem met de huidige modus Oefenen:** je typt en hoort je eigen tekst als zijtoon, maar je oefent tegen niets. De decoder luistert intussen nog naar de radio. Dat is niet logisch.

**Voorstel: Oefenen wordt een simulator.** Er wordt nooit iets gezonden. De decoder luistert niet naar de radio maar naar de simulator. Het geluid komt uit de pc (koptelefoon of luidsprekers). CWLD kan de luidspreker van de radio zelf niet uitzetten: zet de AF-knop van de radio dicht tijdens het oefenen.

Drie oefeningen:

1. **QSO-oefening.** Een gesimuleerd station roept CQ of antwoordt op jouw CQ, met een rapport, naam en QTH. Jij werkt het af met de chips en logt in het testlog. Instelbaar: snelheid, signaalsterkte, ruis en fading (QSB).
2. **Contest-oefening.** Kies een profiel (bijvoorbeeld UBA DX). Stations roepen je aan, soms meerdere tegelijk (pile-up), met verschillende snelheden en sterktes. Je werkt ze met F1 tot F8 en Enter, zoals in een echte wedstrijd. Na afloop zie je je rate, het aantal juist gelogde QSO's en je fouten (verkeerd overgenomen roepletters of nummers).
3. **Paddle-oefening met de radio** (zoals afgesproken, apart na GO). De radio mag dan niet zenden: CWLD zet break-in uit via CI-V en zet het na afloop terug. Het commando moet nog worden nagekeken in de CI-V Reference Guide. Of de zijtoon van de paddle ook via de USB-audio binnenkomt (zodat de decoder hem kan lezen), moet op de radio getest worden.

**Hulp van de decoder aan of uit:** met hulp zie je de gedecodeerde tekst. Zonder hulp oefen je op je gehoor en typ je zelf wat je hoort.

**Morse leren** (letters, Koch-methode, gehoortraining) zit niet in CWLD. Daarvoor verwijst CWLD naar Kristofs eigen oefen-app, met een link in het venster Oefenen en in de handleiding.

De simulator gebruikt de bestaande CW-generator van de testbank (snelheid, ruis, fading, slechte spatiëring), die al in de Rust-code zit.

## 9. Bandscoop

De bandscoop is nu 210 pixels hoog (70 spectrum en 140 waterval). Voorstel: 165 pixels (55 en 110), dat is ongeveer een vijfde lager. Het audio-waterfall rechts blijft zoals het is.

## 10. Website met de laatste versie

- Een moderne website op GitHub Pages, in het Engels en Nederlands (Frans later), uit een map `site/` in de repository, automatisch gepubliceerd door GitHub Actions bij elke push.
- Inhoud: een opvallende openingsafbeelding, wat CWLD kan, schermafbeeldingen, een grote downloadknop die altijd naar de laatste release wijst (de pagina vraagt het aan GitHub, met een gewone link als reserve), systeemeisen, de controlesom (SHA-256), installatiehulp (ook de SmartScreen-melding van Windows), de wijzigingen per versie en een link naar de oefen-app.
- Kristof maakt de afbeeldingen. Claude levert de beschrijvingen (prompts) en de maten.
- Adres: `https://on3vz.github.io/CWLD_DECODE/`. Een eigen domeinnaam kan later.
- Eenmalig in te stellen door Kristof: in de repository op GitHub, Settings, Pages, Source op GitHub Actions zetten. De precieze stappen komen bij de levering.

## 10b. Wat uit het architectuurontwerp (3.11) ook bij het logboek hoort

Bij het nalezen van het architectuurontwerp bleek dat het logboek daar al ruimer was uitgewerkt. Dat komt in ronde C-3, na de contestronde:

- **Stationsprofielen** Thuis (ON3VZ, JO20EE) en Portable (ON3VZ/P, grid per activatie, 5 W, opmerking "5W QRP"), kiesbaar in de statusbalk.
- **Activaties voor fauna en flora** (WWFF, in België ONFF, en eventueel POTA en SOTA): referentie met controle op de vorm `ONFF-0599`, teller van de QSO's, waarschuwing bij middernacht UTC, de juiste WWFF-velden in ADIF en de bestandsnaam voor de WWFF-coördinator.
- **Exportvarianten**: activatie, alles, sinds de laatste export (past bij de werkwijze met PoLo en Log4OM). Elk QSO onthoudt waar het al naartoe ging, zodat niets dubbel verstuurd wordt.
- **Controlevenster vóór export** dat blokkeert bij fouten (verkeerd grid, ontbrekende velden).
- Later: ADIF importeren (bijvoorbeeld uit Log4OM) voor de dupecontrole.

## 11. Correctie

In eerdere berichten stond dat Kristof een tag moet maken voor een nieuwe versie. Dat klopt niet: de release-workflow start vanzelf als een push naar `main` het versienummer in `src-tauri/tauri.conf.json` verandert. Committen en pushen is genoeg.

## 12. Volgorde en modellen

1. **Ronde C-2, contest mode en logboek** (punten 2 tot 7 en 9): **Sonnet**. Het ontwerp ligt vast, de bouw is veel maar overzichtelijk werk.
1b. **Ronde C-3, stationsprofielen, WWFF/ONFF-activaties en exportvarianten** (10b): **Sonnet**.
2. **Ronde O, oefenen** (punt 8, oefeningen 1 en 2): **Opus**. De simulator moet realistisch gedrag nabootsen (wanneer roept een station, hoe reageert het op AGN?, pile-ups) en de generator moet live in de audioketen. Oefening 3 (paddle) daarna, apart na GO.
3. **Ronde G, website**: **Sonnet**.
4. Benchmark tegen MRP40 (**Opus**) helemaal op het einde.

## 12b. Gebouwd in C-2 (0.7.0)

Aannames zolang Kristof niet anders zei: N1MM-toetsen met decoder op Ctrl+F6 en Ctrl+F7, apart testlogboek, bandscoop vast op 165 pixels. Afwijkingen van het voorstel: F7 is NR? en F8 is AGN (F7 stond open); de UBA-sectielijst is niet ingebouwd (bron niet gevonden, aanvullen komt uit eerder gelogde secties); CONTEST_ID is een bewerkbaar veld in het exportvenster, de voorgestelde waarden zijn niet gecontroleerd tegen de ADIF-lijst (de pagina was niet bereikbaar). De slimme AFC van 4 seconden is uitgewerkt als: na een klik in het audio-waterfall wacht de AFC 4 seconden. De berichten achter F1 tot F8 zijn vast.

## 13. Vragen aan Kristof

1. Akkoord met de N1MM-toetsindeling in contest mode, en dat F6 en F7 van de decoder dan Ctrl+F6 en Ctrl+F7 worden?
2. Akkoord met een apart testlogboek (punt 5)?
3. (Beantwoord) De oefen-app staat op https://on3vz.github.io/CWLD/ ; CWLD verwijst daarnaar.
4. Bandscoop naar 165 pixels goed, of liever een instelling (klein, middel, groot)?
5. (Beantwoord) De CWLD-website komt op `on3vz.github.io/CWLD_DECODE/`; `/CWLD/` is de oefen-app en blijft zoals het is.
