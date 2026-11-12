# CWLD: testplan voor je radio's (fase 2 en 3)

Doel: de punten meten die in de bronnen niet duidelijk staan. Alles hieronder is bij het bouwen als **[TEST]** gemarkeerd. Begin altijd met **laag vermogen en een kunstantenne** (dummy load), en laat de app op **LUISTEREN** staan tot een test zendt.

Noteer bij elke test: wat je deed, wat je zag, en of het klopte. Stuur de lijst terug, dan pas ik de drivers aan.

## A. Voorbereiding (eenmalig)

1. **Poort vinden (Windows).** Klik met rechts op de Start-knop, kies Apparaatbeheer, open "Poorten (COM en LPT)". Steek de USB-kabel van de radio erin en kijk welke COM-nummers er bijkomen. De IC-7300 MK II toont er twee.
2. **IC-7300 MK II, menu.** MENU, SET, Connectors, CI-V. Controleer: `CI-V Address` is B6, `CI-V Transceive` staat op ON, en noteer `CI-V Baud Rate` en `CI-V USB Echo Back`.
3. **QMX, menu.** Noteer de firmwareversie. Zet `Audio, Sidetone volume` hoog en `Sidetone abs/rel` op Relative. Zet "Enable Tx" van de decoder op YES.
4. **CWLD.** Instellingen, kies het type radio en de poort, klik Verbinden. In de statusbalk moet "Verbonden" staan en de frequentie van de radio.

## B. IC-7300 MK II

| Nr | Test | Zo doe je het | Wat ik wil weten |
|---|---|---|---|
| I1 | Live frequentie | Draai aan de knop van de radio, wissel van band en van modus. Kijk naar CWLD. | Volgt CWLD binnen een halve seconde? Met CI-V Transceive uit: volgt het dan nog (via de herhaalde vraag)? |
| I2 | Frequentie zetten | Typ in CWLD een frequentie (klik op het getal bovenaan). | Volgt de radio? |
| I3 | Snelheid | Zet de snelheid in CWLD op 12, 20 en 30. Kijk naar de keyer-snelheid op de radio. | Klopt de WPM op de radio met wat CWLD toont? De schaal is een aanname. |
| I4 | Tekst zenden | Zet LIVE aan (laag vermogen, dummy load). Verstuur "CQ CQ DE ON3VZ K". Modus CW, Break-in aan. | Komt de tekst correct uit? Zendt de radio? |
| I5 | Lange tekst | Verstuur een tekst van ongeveer 70 tekens. | Hoor je een gat tussen blokken van 30 tekens? Hoe lang? Kijk of "Wacht om gezonden te worden" de blokken toont. |
| I6 | Noodstop | Verstuur een lange tekst en druk Esc. | Stopt de radio meteen? En komt er daarna geen volgend blok? |
| I7 | Prosign | Verstuur "73 <SK>". | Hoor je SK als een teken (...-.-) of als de letters S en K? |
| I8 | Tweede blok vroeger | Instellingen, Geavanceerd, "Volgend blok vroeger sturen" aan. Verstuur een lange tekst. | Verdwijnt het gat zonder dat tekst verloren gaat? Zo niet, laat het uit. |
| I9 | Paddle tijdens app-tekst | Verstuur een lange tekst en tik met de paddle. | Onderbreekt de paddle de tekst, of wacht de paddle? En doet de wachtrij in CWLD dan het goede? |
| I10 | Meeluistertoon in USB-audio | Neem in Windows (Geluid, Opnemen) het apparaat "USB Audio CODEC" op terwijl je met de paddle zendt (dummy load). | Hoor je je eigen toon in de opname? Hangt het af van het monitor-niveau? |
| I12 | SWR-waarde | Vermogen op het minimum, dummy load. Zend een korte tekst en kijk naar "SWR" bovenaan in CWLD en naar de SWR-meter op de radio. Herhaal met een bekende mismatch (bijvoorbeeld een ontstemde tuner, nooit een open of kortgesloten kabel). | Komt de waarde in CWLD overeen met de radio? De schaal (1,0 / 1,5 / 2,0 / 3,0) is uit de IC-7300-gegevens en moet voor de MK II bevestigd worden. |
| I13 | SWR-alarm | Zet de grens in Instellingen op 1,5. Zend een lange tekst met een slechte belasting op minimaal vermogen. | Hoeveel tijd zit er tussen het begin van de slechte SWR en het stoppen (mik op minder dan een halve seconde)? Stopt de radio echt (zendlamp uit)? Hoor je na het stoppen nog QRX als laatste zending (de radio krijgt eerst `FF` en dan meteen de tekst QRX: komt die er echt uit, of snijdt `FF` hem af)? Is het alarm rood en blijft zenden geblokkeerd tot je bevestigt? Geen vals alarm bij een goede antenne op 20 en 40 m? |
| I14 | SWR met paddle | Zelfde slechte belasting, nu met de paddle. | Komt het alarm? (De app kan een paddle-zending niet stoppen, ze moet alleen waarschuwen.) |
| I11 | Kabel eruit | Trek de USB-kabel tijdens het gebruik. | Meldt CWLD "niet verbonden" zonder te blokkeren? Werkt Verbinden daarna opnieuw? |

## C. QMX

| Nr | Test | Zo doe je het | Wat ik wil weten |
|---|---|---|---|
| Q1 | Identificatie | Verbind. | Komt er een melding "dit lijkt geen QMX"? Zo ja: wat antwoordt de radio op `ID;` (zie Diagnose later)? |
| Q2 | Live frequentie en modus | Draai aan de knop, wissel van modus. | Volgt CWLD? Komt TX/RX mee? |
| Q3 | Snelheid | Zet 12, 20 en 30 WPM. | Klopt het op de radio? |
| Q4 | Tekst zenden | LIVE, dummy load, verstuur "CQ DE ON3VZ K". | Komt het correct uit? Is er een extra pauze op het einde door de spatie na het laatste woord? |
| Q5 | Lange tekst | Verstuur ongeveer 70 tekens. | Komt er een gat tussen woorden? Gaat er tekst verloren (de radio negeert een bericht dat niet past)? |
| Q6 | Noodstop | Verstuur een lange tekst en druk Esc. | Stopt de QMX? Zo niet: stopt `TQ0;` of een druk op de paddle? |
| Q7 | Prosigns | Verstuur "73 <SK>" en "<AR>". | Klinkt SK als een teken? De QMX moet in TS-480-compatibiliteitsmodus staan voor deze tekens. |
| Q8 | Paddle in het gesprek | Tik met de paddle "TEST DE ON3VZ". | Komt het in het gesprek als "Jij (paddle)", zonder dubbels? Hoe snel? Komen er tekens te kort of dubbel? |
| Q9 | Paddle tijdens app-tekst | Verstuur een tekst en tik daarna met de paddle. | Zie je de tekst van de app maar één keer? Wat doet de radio? |
| Q10 | Ontvangen CW in USB-audio | Zet een signaal of baken op, neem "USB Audio CODEC" op. | Zit het ontvangen signaal in de audio, of alleen je eigen meeluistertoon? |
| Q12 | SWR via CAT (`SW;`) | Lees pagina 7 van de QMX CAT-handleiding (cat_1_02_006.pdf of nieuwer). Zend met minimaal vermogen in een dummy load en vergelijk de SWR in CWLD met het QMX-scherm. Controleer ook dat de QMX in ontvangst geen waarde teruggeeft. | Bestaat `SW;` echt in jouw firmware? Komt het antwoord `SWnnn;` snel genoeg voor 100 ms? Is de waarde honderdsten? Zo niet, dan werkt de bewaking op de QMX niet en vertrouw je op de eigen bescherming van de radio. |
| Q11 | Poorten | Kijk hoeveel COM-poorten Windows toont. | Twee? Dan kan Log4OM later de tweede krijgen. |

## C2. Audio en decoder (fase 3)

Eerst I10 en Q10 hierboven: die bepalen of de decoder het ontvangen signaal krijgt.

| Nr | Test | Zo doe je het | Wat ik wil weten |
|---|---|---|---|
| A1 | Ingang vinden | Klik op het tandwiel, scroll naar "Audio en decoder" en open de keuzelijst "Audio-ingang". | Staat "USB Audio CODEC" erbij met "(radio)"? Start het luisteren met "Automatisch"? |
| A2 | Windows-instellingen | Klik met rechts op het luidsprekericoon rechtsonder, kies Geluidsinstellingen, dan Meer geluidsinstellingen. Tab Opnemen, dubbelklik "USB Audio CODEC". Zet op het tabblad Verbeteringen (als het er is) alles uit en kies op het tabblad Geavanceerd 48000 Hz. | Welke standaardindeling stond er? Waren er verbeteringen aan? |
| A3 | Niveau | Zoek een sterk CW-signaal. Kijk naar de niveaubalk onder "Audio en decoder". Pas `USB AF Output Level` op de radio aan (MENU, SET, Connectors) tot de tekst "Niveau is goed" staat. | Welke waarde op de radio gaf een goed niveau? |
| A4 | Waterfall en klik | Kijk naar de waterfall bovenaan. Klik op een CW-signaal. | Zie je de signalen? Springt de rode lijn naar het signaal? Volgt de decoder het? |
| A5 | Decoderen | Laat een sterk en een zwak signaal een minuut lopen. Vergelijk met wat je zelf hoort en met de decoder van de radio. | Hoeveel tekens gingen mis? Toont de zekerheidsmeter een lage waarde bij het zwakke signaal? |
| A6 | Opname | Klik bovenaan op "OPNAME", laat een QSO een minuut lopen, klik "Stop". Open Documenten\CWLD in Verkenner. | Staat het bestand er, met datum, tijd en frequentie in de naam? Speelt het af? |
| A7 | Vergelijking met CwGet | Laat CwGet hetzelfde WAV-bestand decoderen. Laat CWLD het ook decoderen: Instellingen, "Een WAV-bestand decoderen", plak het volledige pad, klik Decoderen. | Welke tekst geven beide? Schrijf erbij wat er werkelijk gezonden werd als je dat weet. |
| A8 | Eigen tekst niet dubbel | LIVE, dummy load, laag vermogen: verstuur een chip. | Verschijnt de tekst maar één keer in het gesprek? |
| A9 | Paddle via audio (Icom) | Sein met de paddle "TEST DE ON3VZ". | Komt het als "Jij (paddle)"? Dat lukt alleen als I10 toonde dat de meeluistertoon in de USB-audio zit. |
| A10 | Kabel los | Trek tijdens het luisteren de USB-kabel uit. | Verschijnt "Audio uit" bovenaan, zonder dat CWLD vastloopt? Start het weer na "Luisteren starten"? |
| A11 | Audio-ingang kiezen | Kies in de lijst "Audio-ingang" de USB Audio CODEC. | Staat er onder "Luistert" nu de ingang van de radio, zonder dat je nog ergens op hoeft te klikken? |
| A12 | Zoom | Klik rechtsboven op de waterfall op 1x, 2x en 4x. Klik op een signaal. | Zie je de toonhoogte (rode lijn) en de getallen onderaan? Springt de rode lijn naar het signaal bij elke zoom? |
| A13 | Drempel | Stem af op een stille plek met alleen ruis. Zet in de instellingen "Drempel voor signalen" op 2 en daarna op 5. | Komt er bij 2 nog tekst uit de ruis? Verdwijnt die bij 5? Verdwijnt een zwak maar echt signaal ook? Wat doet de indicator "Signaal"? |
| A14 | Gesprekken | Draai 2 kHz weg met de draaiknop. Klik op "Nieuw QSO". Klik daarna op het eerste tabblad. | Komt er een nieuw tabblad bij de frequentiesprong? Blijft het oude gesprek leesbaar, met zijn eigen roepletters en naam? Staat het er nog na CWLD sluiten en weer openen? |

## D. Wat ik met de uitkomst doe

- Klopt alles, dan blijft de driver zoals hij is.
- Klopt iets niet, dan pas ik precies dat aan en voeg ik een test toe met wat de radio echt deed.
- De punten I10, Q10 en Q8 bepalen hoe de decoder in fase 3 aan zijn audio komt.

## A15. Bandscoop (0.4.0)

1. Verbind de IC-7300 MK II zoals altijd. Binnen een paar seconden moet bovenaan de bandscoop verschijnen met hetzelfde beeld als op de radio.
2. Zet op de radio de span op ±25 kHz en daarna op ±5 kHz. Controleer dat de frequenties onder de scoop (bv. 7000.0 tot 7050.0 kHz bij ±25 kHz rond 7025) kloppen met het scherm van de radio. Wijkt het een factor twee af, meld het: dan lezen we de span verkeerd.
3. Klik op een signaal in de scoop. De radio moet naar dat signaal gaan en je hoort de toon.
4. Draai aan de knop van de radio: de rode lijn en het beeld moeten meelopen.
5. Zend een korte CQ in LIVE en kijk of de scoop of het zenden hapert. Meld het als de radio commando's weigert (melding in de app).
6. Klik op Bandscoop verbergen en weer tonen. Herstart de app: de keuze moet onthouden zijn.

## A16. Codes en afkortingen (0.4.1)

Geen radio nodig.
1. Klik naast Verzenden op **Q** (of druk Ctrl+Q). Verwacht: rechts schuift een paneel open.
2. Typ `qth` in het zoekvak. Verwacht: QTH staat bovenaan met uitleg. Typ `locatie`: QTH staat in de lijst.
3. Klik op de knop QTH. Verwacht: `QTH ` staat in het zendvak. Klik op de link `QTH?` bij Als vraag: `QTH? ` komt erbij.
4. Kies het tabblad Prosigns en klik op `<AR>`. Verwacht: `<AR>` in het zendvak. `<CL>` is grijs.
5. Druk Esc. Verwacht: het paneel sluit (niets wordt gezonden).


## A17. Instellingen, venster en status (0.5.0)

1. Start CWLD. Verwacht: het venster is meteen maximaal.
2. Open het tandwiel. Verwacht: links een menu met rubrieken, niet meer één lange lijst. Klik door alle rubrieken.
3. Zet de radio uit (of trek de USB-kabel los). Verwacht binnen ongeveer 5 seconden: de status rechtsboven wordt "Radio zoeken" met een knipperend bolletje, niet "Verbonden".
4. Zet de radio weer aan (of steek de kabel terug). Verwacht binnen ongeveer 10 seconden: de status wordt vanzelf weer "Verbonden", zonder dat je op Verbinden drukt. Meld wat je zag als dit niet zo gaat.

## A18. Pile-up en rapport (0.5.0)

1. Zet LIVE of OEFENEN aan (OEFENEN is veilig). Druk F1, F2, F3 en F4 en kijk wat er in het gesprek verschijnt.
2. Verwacht: F1 roepletters, F2 twee keer, F3 `R 5NN TU`, F4 `AGN?`.
3. Instellingen, Zenden: kies 599. Open een nieuw gesprek met Nieuw QSO en druk F3. Verwacht: `R 599 TU`.
4. In LUISTEREN doen de toetsen niets.

## A19. Tabjes (0.5.0)

1. Draai de radio naar een andere band en wacht 5 seconden. Er komt een nieuw tabje. Klik terug op het eerste tabje. Verwacht: de radio gaat terug naar die frequentie, en er komt geen extra tabje bij.
2. Doe dat tijdens het zenden in OEFENEN of LIVE. Verwacht: alleen tonen, de radio blijft.
3. Zet bij Weergave en taal de sluittijd op 1 minuut, laat een gesprek met alleen ontvangen tekst liggen en wacht. Verwacht: het sluit na ongeveer een minuut (de controle loopt om de 30 seconden). Een gesprek waarin je iets zond sluit niet.
4. Klik op Alles sluiten. Verwacht: een vraag als er gesprekken met gezonden tekst zijn, daarna één leeg gesprek.

## A20. Codes en afkortingen in het gesprek (0.5.0)

1. Zet de muis op TNX, UR of QTH? in een ontvangen of gezonden bericht. Verwacht: een tooltip met de betekenis.
2. Open onder het gesprek de lijst Codes in dit gesprek. Verwacht: elke code één keer, met uitleg. Klik op een code: ze komt in het zendvak.
3. Open het Q-paneel (Ctrl+Q), tab Eigen. Voeg `TNX2U` toe met een betekenis. Verwacht: ze staat in de lijst, en in een bericht met dat woord komt een tooltip.

## A21. Decoder bijstellen (0.5.0)

1. Bekijk de piekmeter bij een sterk signaal. Verwacht: tussen 5 en 90 procent, groen. Zet het audioniveau van de radio te hoog: de balk wordt rood boven 90 procent en er staat een waarschuwing.
2. Beweeg de versterking. Verwacht: de balk volgt (de opname niet).
3. Klik Breed. Verwacht: de twee rode lijnen in het waterfall staan verder uit elkaar (ongeveer het dubbele).
4. Klik Sterkste (F6) terwijl er een ander signaal op de band staat. Verwacht: de decoder gaat naar het sterkste.
5. Zet bij Decoder een vaste snelheid die niet bij het station past. Verwacht: slechte tekst. Zet terug op 0 (automatisch).
6. Zet AFC uit met F7 en weer aan.

## A22. Contest (0.5.0)

1. Klik onder de chips op Contest. Verwacht: er verschijnt een chip Uitwisseling (`{CALL} 5NN 001`) en bij de Contest-knop staat 001. Kijk met de muis boven de chip: de tooltip toont de tekst.
2. Zet het nummer op 5, zet contest uit en weer aan. Verwacht: de vraag Doorgaan of Opnieuw bij 1.

## A23. Installer sluit CWLD af (0.5.1)

1. Laat CWLD draaien, met de radio verbonden en audio aan. Start het installatiebestand van de nieuwe versie (of gebruik de updateknop).
2. Verwacht: het installatievenster toont "Stopping CWLD..." en eventueel "Waiting for CWLD to close (1)", en daarna "CWLD has stopped". De installatie loopt door zonder de melding "Failed to kill CWLD".
3. Lukt het niet, dan staat er in het venster hoe je cwld.exe in Taakbeheer beëindigt (tabblad Details, klik rechts op cwld.exe, Taak beëindigen). Meld mij wat er in het venster stond.

## A24. Simulator is zichtbaar (0.5.1)

1. Start CWLD met een vers instellingenbestand (of kies in Instellingen, rubriek Radio en SWR, bij Type radio de Simulator). Verwacht: rechtsboven staat oranje "Simulator, geen radio" in plaats van groen "Verbonden", en onder de statusbalk staat een oranje balk "Nog geen radio gekozen" met een knop Radio kiezen. De knop opent de instellingen.
2. Kies je IC-7300 MK II, kies de COM-poort en klik Verbinden. Verwacht: de oranje balk en het oranje label verdwijnen. Bij een ingeschakelde radio staat er groen "Verbonden" met de naam van de radio.
3. Zet de radio uit. Verwacht binnen enkele seconden "Radio zoeken" (geen groen "Verbonden"). Zet hem weer aan: hij verbindt vanzelf. Dit moet kloppen als Type radio op IC-7300 MK II staat. Meld het als het anders is.

## A25. Wedstrijdprofielen (0.6.0)

1. Klik onder de chips op Contest. Verwacht: er verschijnt een keuzelijst (standaard "Geen profiel"). Kies UBA DX Contest CW. Verwacht: een gele opmerking over de regels, een tweede keuzelijst "Vanuit België", een chip TEST UBA en een chip Uitwisseling.
2. Wijs met de muis naar de chip Uitwisseling. Verwacht: de tooltip toont `{DX-CALL} 5NN 001 WLD` (met de roepletters in het QSO-paneel). Kies in de tweede lijst "Vanuit het buitenland". Verwacht: de tooltip toont de uitwisseling zonder WLD.
3. Zet de radio in Luisteren en klik op de chip. Verwacht: er wordt niets gezonden. Zet CWLD in Oefenen en klik op de chip. Verwacht: je hoort de uitwisseling in de zijtoon.
4. Open Instellingen, rubriek Contest. Verwacht: een lijst met profielen, een editor en onder Eigen variabelen `SEC` WLD, `ZONE` 14, `ITU` 27 en `PWR` 20. Verander `SEC` en kijk of de chip Uitwisseling mee verandert.
5. Kies CQ WW DX CW in de lijst, klik op Kopiëren, zet Verkorte cijfers aan en klik op Profiel opslaan. Kies het profiel op het hoofdscherm. Verwacht (bij een profiel met `{NR}`, zoals CQ WPX): het volgnummer toont als TT1.
6. Probeer bij Eigen variabelen een variabele `CALL` of `NR` te maken. Verwacht: een foutmelding.
7. Exporteer al je profielen (knop onderaan). Verwacht: een bestand `cwld-contest-backup.json` dat je kunt openen. Verwijder een eigen profiel en importeer het bestand. Verwacht: het profiel is terug. Meld mij of de download in de app werkt (op Windows) en waar het bestand terechtkomt.
8. Lees de opmerking bij CQ WW DX. Verwacht: de waarschuwing dat een CW-decoder als hulpmiddel telt.

## A26. Contest mode: een QSO in Run (0.7.0)

1. Zet CWLD in Oefenen (je hoort de zijtoon, er wordt niets uitgezonden). Klik Contest en kies UBA DX Contest CW. Controleer bij Instellingen, Station of je roepletters ingevuld zijn.
2. Druk Enter met een lege invoerregel. Verwacht: `TEST UBA ON3VZ` in het gesprek.
3. Typ `PA3ABC` in Call en druk Enter. Verwacht: `PA3ABC 5NN 001 WLD`. Het nummer naast Log blijft 001.
4. Druk nog eens Enter zonder iets in te vullen. Verwacht: `AGN`. Het nummer is nog 001.
5. Vul bij Nr ontv. `12` in en druk Enter. Verwacht: `TU ON3VZ`, het QSO staat in het logpaneel, het nummer is 002 en de invoerregel is leeg.
6. Typ `ON4XYZ`. Verwacht: het veld UBA-sectie wordt actief (alleen bij Belgische stations).

## A27. Contest mode: S&P, dupes, waarschuwing (0.7.0)

1. Druk Ctrl+R. Verwacht: S&P is gekleurd. Typ `DL1ABC`, Enter. Verwacht: alleen `ON3VZ`. Vul `5` in bij Nr ontv. en druk Enter. Verwacht: je uitwisseling gaat uit en het QSO wordt gelogd.
2. Typ `DL1ABC` nog eens. Verwacht: een rode DUPE met tijd en band.
3. Typ `PA3XYZ`, druk F2 (uitwisseling) en typ daarna `PA3QQQ` zonder te loggen. Verwacht: een waarschuwing met het gezonden nummer en PA3XYZ.
4. Controleer dat het nummer nog hetzelfde is als voor de F2.

## A28. Testmodus (0.7.0)

1. Klik Test. Verwacht: oranje rand, TEST in de inforegel, het nummer begint bij 001. Log twee QSO's. Verwacht: het nummer gaat naar 003 en het logpaneel toont ze.
2. Zet Test uit. Verwacht: het echte nummer staat nog op wat het was en de testQSO's staan niet in het logpaneel.
3. Zet Test aan, klik Testlog wissen. Verwacht: het logpaneel is leeg en het testnummer is weer 001.

## A29. Klikken en voorstellen (0.7.0)

1. Klik in een ontvangen bericht op een woord. Verwacht: een menu met Roepletters, Rapport, Nummer, Extra veld, Naam en QTH. Kies Nummer. Verwacht: het woord staat in Nr ontv.
2. Zend je uitwisseling en laat de decoder een antwoord als `5NN 023 ANR` horen (bijvoorbeeld met een opname). Verwacht: een grijs voorstel onder de invoerregel. Druk Tab in een leeg veld: het wordt overgenomen.

## A30. Export, logpaneel, bandscoop, AFC (0.7.0)

1. Klik in het logpaneel op Exporteren, kies Alle QSO's, controleer CONTEST_ID en klik Exporteren. Verwacht: "n QSO's opgeslagen in ..." met een pad onder Documenten, CWLD. Open het bestand in Kladblok: ADIF-velden zoals `<CALL:6>PA3ABC`. Importeer het in Log4OM en meld hoe de velden aankomen (vooral STX, SRX en CONTEST_ID).
2. Verbeter een QSO met de knop Verbeter en verwijder er een (twee stappen).
3. Zet CWLD af en weer aan. Verwacht: de QSO's staan er nog. Controleer dat naast `logbook.db` in de CWLD-map van de app een `logbook.db.bak` staat (map %APPDATA%\io.github.on3vz.cwld).
4. Bandscoop: is hij lager (165 pixels) en nog goed leesbaar?
5. Klik in het audio-waterfall op een signaal terwijl AFC aan staat. Verwacht: de toon blijft ongeveer 4 seconden waar je klikte en volgt daarna het signaal. Meld of dat prettig werkt.


## A31. Profielen en activaties (0.8.0)

1. Controleer in de statusbalk de keuzelijst met profielen. Kies Portable. Verwacht: ON3VZ/P als roepletters in de berichten (variabele {MYCALL}).
2. Klik op Nieuwe activatie, laat het grid leeg en klik Starten. Verwacht: melding dat het grid ontbreekt.
3. Vul ONFF599 in. Verwacht: rode rand en tekst dat het niet op ONFF-0599 lijkt. Vul ONFF-0599 en een grid in en start. Verwacht: paneel met referentie en grid in grote letters, de profielkeuze is vergrendeld.
4. Log twee QSO's, een ervan met een andere roepletters. Verwacht: teller 2 QSO's en 2 unieke. Log er een met dezelfde roepletters: unieke blijft gelijk.

## A32. Export met controle (0.8.0)

1. Open Exporteren, kies Huidige activatie en klik Controleren. Verwacht: Controle geslaagd, of een lijst met waarschuwingen. De knop Exporteren is pas actief na de controle.
2. Exporteer. Verwacht: bestand met naam zoals `ON3VZ-P @ ONFF-0599 datum.adi` in Documenten, CWLD. Open het in Kladblok: MY_GRIDSQUARE, MY_SIG, MY_SIG_INFO, STATION_CALLSIGN en TX_PWR staan erin.
3. Open Exporteren en kies Alleen QSO's die nog niet geëxporteerd zijn. Verwacht: 0 QSO's of alleen de nieuwe.
4. Importeer het bestand in Log4OM en in PoLo en stuur me beide exportbestanden (open punt: veld voor veld vergelijken).
5. Instellingen, Weergave: zet de hoogte van de bandscoop op klein, midden en groot. Controleer dat contestmodus de eigen keuze volgt.

## A33. Oefenen tegen gesimuleerde stations (0.9.0)

1. Zet de AF-knop van de radio dicht. Klik OEFENEN. Verwacht: paneel Oefenen rechts.
2. Kies QSO: een station roept CQ en klik Oefening starten. Verwacht: na ongeveer 1 seconde hoor je via de pc een CQ en de decoder toont hem. Controleer dat de radio niet zendt (geen TX-lampje, vermogensmeter stil).
3. Antwoord met de chip Antwoord. Verwacht: het station geeft rapport, naam en QTH. Zend je rapport, het station sluit af met 73. Log het QSO.
4. Klik Stoppen en score tonen. Verwacht: 1 juist van 1 gelogd. Daarna luistert de decoder weer naar de radio (zet de AF-knop terug open).
5. Kies Contest (Run), profiel CQ WPX, 3 aanroepers, 5 minuten. Druk F1. Verwacht: een of meer stations roepen, op licht verschillende toonhoogtes. Werk er enkele af met Enter en F2/F3. Probeer NR? (F7) en een deel van een roepletters met een vraagteken.
6. Vink Hulp van de decoder uit. Verwacht: ontvangen tekst wordt puntjes.
7. Schakel tijdens een oefening naar LUISTEREN. Verwacht: de oefening stopt, de stations zwijgen.
8. Meld: is het volume goed, klinken de stations natuurlijk, kraakt of hapert het geluid, en is de snelheid waarop stations reageren realistisch?

## A34. Begeleide eerste start (0.10.0)

1. Open Instellingen, Updates en info, klik op Begeleide start. Verwacht: venster met vier stappen. Esc sluit het niet, Overslaan wel.
2. Stap 1: kies Icom IC-7300 MK II terwijl de radio aan staat. Verwacht: de poort van de CI-V (Serial Port A) is al gekozen. Klik op Test de verbinding. Verwacht: Verbonden en de frequentie in kHz.
3. Trek de USB-kabel uit en klik op Opnieuw zoeken. Verwacht: de lijst met drie controles. Steek de kabel terug.
4. Stap 2: zet CI-V Transceive op OFF in de radio, draai aan de knop. Meld wat het venster toont (volgt de frequentie toch dankzij het opvragen?). Zet het daarna op ON en controleer dat de melding Goed verschijnt.
5. Stap 3: de radio-ingang staat voorgesteld. Klik op Begin te luisteren. Verwacht: niveaubalk beweegt bij een signaal. Zet in Windows de microfoon-privacy uit en meld wat er gebeurt.
6. Stap 4: vul je gegevens in en kijk naar de stille test. Klik op Klaar. Verwacht: niets gezonden, CWLD start voortaan verbonden en luisterend.
7. Wis de instellingen (map %APPDATA%\io.github.on3vz.cwld, bestand settings.json hernoemen) en start CWLD. Verwacht: het venster opent vanzelf.

## A35. Installer zonder "Failed to kill CWLD" (0.10.1)

1. Sluit CWLD. Start het installatieprogramma van 0.10.1 (dubbelklik op het bestand dat eindigt op -setup.exe).
2. Klik in het installatievenster op Show details (of Details tonen) om de regels te zien.
3. Verwacht: "CWLD has stopped" en daarna gewoon verder installeren. Staat er "The old cwld.exe is in use by another program", stuur dan een schermafbeelding van de regel "In use by: ...". Dat is het programma dat het bestand vasthield.
4. Herhaal met CWLD open: het installatieprogramma sluit CWLD en installeert.
5. Herhaal via de update in CWLD zelf (zodra 0.10.2 of later bestaat).

## A36. Meer ruimte op het scherm (0.10.2)

1. QSO-paneel: staat de lege tekst weg? Houd de muis boven het lege roepletterveld (klik Nieuwe QSO): verschijnt de uitleg als tooltip? Staan RST en RST ontv. naast elkaar en zijn ze groot genoeg om te lezen?
2. Audio-paneel: staat Afstellen dicht? Klik erop: komen Zwak, Midden, Sterk, Smal, Breed, Sterkste en AFC en de gain-schuif tevoorschijn? Sluit CWLD en start opnieuw: onthoudt hij je keuze? Werken F6 en F7 ook als het dicht staat?
3. Activatie: de groene knop Activatie staat rechts naast Alles sluiten. Start een activatie (ONFF-0599 en een grid): toont de knop de referentie en het aantal QSO's? Log een QSO: gaat de teller omhoog? Klik op de knop: grid in grote letters, Activatie beëindigen werkt, Sluiten werkt.
4. Meld of de groene kleur van de knop past, of dat je een andere tint wilt.

## A37. UTC overal (0.10.3)

1. Vergelijk de UTC-klok in de statusbalk met een UTC-klok (bijvoorbeeld time.is/UTC). Verwacht: gelijk op de seconde na. In Belgie is dat in de winter een uur, in de zomer twee uur achter op je pc-klok.
2. Log een QSO en kijk naar de tijd in het logpaneel en naar de tijd boven berichten in het gesprek. Verwacht: beide UTC.
3. Exporteer en open het ADIF-bestand in Kladblok: TIME_ON moet de UTC-tijd van het QSO zijn.

## A38. Noodstop en foutteken (0.10.4)

Doe dit op een dummy load of met laag vermogen, in LIVE.

1. Zend een lange tekst (bijvoorbeeld CQ CQ CQ DE ON3VZ ON3VZ K). Druk een keer op Esc. Verwacht: het zenden stopt meteen, de wachtrij is leeg, de melding zegt dat een tweede Esc ook QRX QRX zendt.
2. Zend opnieuw en druk twee keer snel op Esc. Verwacht: stop en daarna QRX QRX in CW, ook in het gesprek.
3. Zend opnieuw, druk Esc, wacht drie seconden en druk nog eens. Verwacht: niets wordt gezonden.
4. Druk Esc zonder dat er iets gezonden wordt, twee keer. Verwacht: niets wordt gezonden.
5. Zend en klik op Stop + QRX. Verwacht: hetzelfde als stap 2.
6. Typ een tekst in het zendvak, klik op Fout. Verwacht: acht puntjes (luister mee), de tekst in het zendvak blijft staan. Meld of de radio de acht puntjes goed geeft (^HH) of dat er een pauze of een ander teken te horen is.
7. Meld of de radio direct na de stop de QRX goed zendt, of dat de eerste letter wegvalt.

## A39. Logkoppeling Log4OM (0.11.0)

1. Stel Log4OM in zoals in sectie 23 van de handleiding (UDP INBOUND, poort 2333, ADIF_MESSAGE) en zet de koppeling in CWLD aan.
2. Log een echt QSO in CWLD. Verwacht: het staat meteen in Log4OM, met call, datum en tijd (UTC), band, frequentie, RST en eigen roepletters. Meld welke velden ontbreken of verkeerd staan.
3. Klik op Laatste QSO opnieuw sturen. Verwacht: Log4OM meldt een dubbel QSO of negeert het, zonder crash.
4. Log een testQSO (testlogboek). Verwacht: er komt niets in Log4OM.
5. Zet de koppeling uit en log nog een QSO. Verwacht: er komt niets in Log4OM.
6. Meld of Log4OM het bericht pas na een druk op een knop verwerkt, en of de poort 2333 bij jou al door een ander programma gebruikt werd.
7. Optioneel, N1MM-formaat: stuur naar een luisterend programma en meld of het contactinfo-bericht begrepen wordt.

## A40. Radio stil in OEFENEN (0.13.1, squelch in plaats van volume)

Bevindingen ON3VZ met 0.12.0 en 0.13.0: AF op 0 maakte de band stil, maar ook de zijtoon op de USB, dus paddle oefenen werkte niet. Met AF op 20 procent en de SQL-knop helemaal rechtsom was de band stil en werkte de herkenning. Daarom sluit 0.13.1 de squelch.

1. Verbind de IC-7300 MK II, LUISTEREN, AF-knop op ongeveer 20 procent, SQL-knop linksom (open), BK-IN aan (semi).
2. Kijk rechts in het oefenpaneel na de overstap naar OEFENEN: "Radio meldt: volume ..., squelch ..., break-in ...". Noteer de waarden.
3. Verwacht binnen een seconde: de band valt stil, BK-IN gaat uit op het scherm van de radio, squelch "dicht", en "Radio stil".
4. Sein met de paddle. Verwacht: de radio zendt niet (geen TX-lampje) en je hoort de zijtoon.
5. Draai de SQL-knop open tijdens OEFENEN. Verwacht: CWLD laat hem zo.
6. Klik LUISTEREN. Verwacht: squelch terug zoals hij was, BK-IN terug aan, AF ongewijzigd.
7. Staat er "niet gemeld" bij een waarde, meld dat: dan antwoordt de radio niet op dat commando.

## A41. Paddle oefenen (0.12.0, opnieuw met 0.13.2)

0. Op de radio eenmalig: MENU, SET, Connectors, USB AF/IF Output, "AF Beep/Speech... Output" op ON (bevestigd door ON3VZ: daarna komt de zijtoon op de USB-audio). AF-volume op ongeveer 20 procent.
1. OEFENEN, audio-ingang van de radio actief, klik "Start paddle oefenen" met Tekengroepen. Verwacht: een nieuw tabblad "Oefenen".
2. Sein de groepen. Verwacht: tekst onder "Gehoord", na twee seconden stilte de beoordeling.
3. Sein met opzet een te korte pauze tussen tekens. Verwacht: de tip "De tekens lopen in elkaar".
4. Meld de gemeten WPM tegenover de keyer-snelheid van de radio.
5. Blijft "Nog geen zijtoon gehoord" staan: doe de stappen in handleiding sectie 24 en meld wat werkte.

## A42. Decoder pauzeert bij SSB en spraak (0.12.0)

1. LUISTEREN, radio op CW op een CW-signaal: tekst komt zoals altijd.
2. Zet de radio op USB. Verwacht: geen tekst meer, melding "Decoder gepauzeerd: de radio staat op USB".
3. Zet terug op CW. Verwacht: de melding verdwijnt en het decoderen start weer.
4. Radio op CW, draai naar een SSB-station (bijvoorbeeld 7,150 MHz). Verwacht: "Spraak of een breed signaal" en geen of bijna geen letters.
5. Meld een geval waarbij een echt CW-signaal ten onrechte als spraak gezien werd (frequentie en tijd, liefst met een opname via OPNAME).

## A43. QRZ.com opzoeken (0.13.0)

1. Instellingen, Logkoppeling, QRZ.com opzoeken: aanvinken, gebruikersnaam en wachtwoord invullen, "Opslaan en testen". Verwacht: "Inloggen werkt" (met of zonder abonnement-melding).
2. Sluit CWLD en open het opnieuw. Verwacht: het wachtwoordveld toont "Opgeslagen" en een test zonder opnieuw typen werkt.
3. Open Windows Referentiebeheer (Start, typ "Referentiebeheer", Windowsreferenties). Verwacht: een item "CWLD QRZ.com" met je gebruikersnaam. Het wachtwoord staat niet in settings.json (open dat bestand in kladblok en zoek).
4. Typ in het QSO-paneel een bekende roepletterreeks. Verwacht: na een korte pauze verschijnen Naam en QTH, met een regel "Van QRZ.com".
5. Typ zelf een andere naam in, wis de roepletters en typ ze weer. Verwacht: je eigen naam blijft staan.
6. Typ een niet bestaande roepletterreeks. Verwacht: geen invulling, geen foutmelding.
7. Trek de netwerkkabel eruit. Verwacht: één melding dat QRZ.com niet bereikbaar is, loggen werkt gewoon.
8. Probeer een verkeerd wachtwoord. Verwacht: "QRZ.com weigerde de aanmelding".
9. Meld ook: welke velden een gratis account teruggeeft (naam en plaats ja of nee).

## A44. QRZ.com bij getypte roepletters (0.13.1)

1. QRZ.com aan (zie A43). Typ in het QSO-paneel een bekende roepletterreeks met het toetsenbord.
2. Verwacht: na een korte pauze Naam en QTH ingevuld, met de grijze regel "Van QRZ.com" voor precies die roepletters.
3. Typ daarna andere roepletters. Verwacht: de grijze regel verdwijnt meteen, naam en QTH van het vorige station verdwijnen en worden vervangen door die van het nieuwe station.
4. Typ zelf een naam en wissel dan de roepletters. Verwacht: jouw getypte naam blijft staan.

## A45. Koch-methode (0.14.0)

1. Zet OEFENEN aan. Verwacht: rechts een kaart "Tekens leren (Koch)" met alle 40 tekens aan.
2. Sleep het schuifje naar 5. Verwacht: alleen K M R S U zijn gekleurd, U heeft een rand.
3. Klik op "Start luisteren". Verwacht: een toon uit de pc, nooit uit de radio, vier groepen van vijf tekens uit K M R S U, snelle tekens met lange pauzes.
4. Typ wat je hoort (kleine letters mogen) en druk Enter. Verwacht: foute tekens rood onderstreept met jouw invoer eronder, een percentage juist.
5. Druk Esc tijdens het typen. Verwacht: de oefening stopt.
6. Zet totale snelheid op 20 en tekensnelheid op 20. Verwacht: gewone morse zonder extra pauzes. Zet tekensnelheid op 15 en totale snelheid op 18: totale snelheid wordt 15.
7. Doe drie rondes met minstens 90 procent juist. Verwacht: een voorstel "Voeg A toe". Zonder klik blijft het aantal 5.
8. Start de paddle oefening "Tekengroepen" (radio in OEFENEN). Verwacht: alleen K M R S U (en A als je die toevoegde).
9. Kies "Roepletters" met alleen letters. Verwacht: melding dat er te weinig echte roepletters passen, en tekengroepen.
10. Meld: is 20 en 12 WPM prettig als begin, en klopt de toon qua volume?

## A46. Paddle oefenen met de decoder op een andere toon (0.15.0)

1. LUISTEREN: klik in de waterfall op een signaal dat duidelijk niet op 600 Hz zit (bijvoorbeeld 450 of 800 Hz) en zet "Sterkste" uit.
2. Zet OEFENEN aan en start een paddle-oefening (radio zoals in A40: AF ongeveer 20 procent, AF Beep/Speech Output ON).
3. Sein de tekst. Verwacht: "Gehoord" vult zich en de oefening wordt gescoord, ook al staat de decoder niet op je zijtoon.
4. Meld als het toch niet werkt: staat er een balkje in de audio, wat is de CW-pitch van de radio (MENU, SET, of de PITCH-knop), en wat toont de regel "Radio meldt".

## A47. Decoder op slordige morse (0.15.0)

1. Zoek een QSO waar iemand zijn naam of QTH spelt met de letters ver uit elkaar. Verwacht: de naam komt als één woord, niet als losse letters.
2. Zoek een station dat CQ en DE aan elkaar seint. Verwacht: `CQ DE` als twee woorden.
3. Let op de vertraging: de lichtere voorlopige tekst komt meteen, de definitieve tekst ongeveer vijf tekens later. Meld als dat stoort.
4. Leest CWLD iets fout dat jij wel kon volgen: druk OPNAME en stuur de opname met wat er gezonden werd. Ook voor de zeven opnames van 10 november is de juiste tekst nog welkom.

## A48. Zenden oefenen vanuit de Koch-kaart (0.16.0)

1. Zet OEFENEN aan. Kies in de kaart "Tekens leren (Koch)" je tekens (bijvoorbeeld de eerste 6).
2. Klik onder "Zenden (paddle)" op "Sein groepen". Verwacht: een tabblad "Oefenen" met groepen uit alleen jouw tekens. Liep het luisteren nog niet, dan start het vanzelf.
3. Sein de tekst met de paddle. Verwacht: "Gehoord" vult zich terwijl je seint en na twee seconden stilte komt de score.
4. Werkt het niet, meld dan: beweegt het audiobalkje als je seint, wat staat er bij "Radio meldt", en welke versie staat er onderaan in Instellingen.

## A49. Schakelaar Fading (0.16.1)

1. Kijk in de kaart Audio op het hoofdscherm (rechts). Verwacht: onder de piekmeter "Fading (QSB)" met "Doorlezen" en "Streng", en een regel uitleg eronder.
2. Zoek een station met duidelijke QSB. Laat een halve minuut op "Doorlezen" staan en zet dan op "Streng". Verwacht: bij "Streng" breken woorden vaker in stukken.
3. Zoek een heel zwak, stabiel station. Vergelijk beide standen. Meld welke stand daar beter leest.
4. Sluit CWLD en start opnieuw. Verwacht: de gekozen stand staat er nog.

## A50. DX-cluster verbinden (0.18.0)

1. Instellingen (tandwiel), Station: roepletters ingevuld? Ga naar DX-cluster en klik **Test de clusters**. Verwacht: een lijst met per node "antwoordt in ... ms" of "geen antwoord". Noteer welke antwoorden, vooral ON0DXK.
2. Laat Cluster op Automatisch en klik **Verbinden**. Verwacht: "Verbonden met ... (CC Cluster of DX Spider)". Na een tot twee minuten staan er spots.
3. Sluit de instellingen. Kijk naar de bandscoop op een CW-band. Verwacht: labels met roepletters, in de balk eronder "<node>: N spots".
4. Meld: welke node werd gekozen, welke software er stond, en of de spots in de buurt van echte signalen op de scoop staan.
5. Zet "Ook spots van skimmers tonen" uit, klik **Opnieuw verbinden**. Verwacht: veel minder spots. Zet het weer aan. Meld of het aantal spots verschilt.

## A51. Kleuren en dupes (0.18.0)

1. Log een QSO met een station dat als groen label op de scoop staat. Verwacht: het label wordt meteen rood en doorgestreept.
2. Zoek een station dat je op een andere band werkte. Verwacht: oranje op een band waar je het nog niet werkte.
3. Zet contestmodus aan met een lopende contest. Verwacht: alleen QSO's van deze contest (en dit nummer van de contest) maken labels rood.
4. Klik op een label. Verwacht: de radio stemt af op die frequentie en er wordt niets gezonden.

## A52. Cluster uitvallen en commando's (0.18.0)

1. Zet tijdens verbinding het netwerk even uit (wifi uit). Verwacht: status "offline" met de reden, labels blijven staan tot ze verlopen. Zet het netwerk weer aan. Verwacht: CWLD verbindt vanzelf opnieuw, met de volgende node als je op Automatisch staat.
2. Open in Instellingen, DX-cluster, onderaan het Clustervenster. Typ `sh/dx 5`. Verwacht: de laatste vijf spots verschijnen in het venster.
3. Typ `dx 7030 TEST`. Verwacht: een melding dat het commando niet mag, en er gaat niets naar de cluster.
4. Meld: werkt `set/skimmer` bij jouw node (komen er na het commando meer spots), en welke bijzonderheden de node meldt.

## A53. Rustige labels en spots als hint (0.18.1)

1. Verbind met de cluster en kijk naar de bandscoop. Verwacht: kleine afgeronde labels in grijs met een gekleurd bolletje, geen felgroene tekst meer. Meld of de labels zo goed leesbaar zijn, of nog te druk of juist te flets.
2. Log een station dat een groen bolletje heeft. Verwacht: het label vervaagt en het bolletje wordt grijs.
3. Stem af op een CW-station dat als label op de scoop staat (klik op het label) en laat CWLD een paar minuten meelezen. Meld of de roepletters van dat station goed doorkomen, ook bij QSB.
4. Zie je dat CWLD een roepletter "verbetert" naar die van een spot terwijl er iets anders gezonden werd: druk OPNAME, noteer de roepletters van de spot en wat er echt gezonden werd, en stuur de opname.

## A54. Nieuwe bandscoop (0.18.2)

1. Verbind de radio en kijk naar de bandscoop met jouw gewone breedte (bijvoorbeeld 200 kHz). Verwacht: bovenaan de labels, daaronder een duidelijk spectrum met pieken, dan de frequentieschaal en een waterval die de rest vult.
2. Verwacht: de ruis ligt onderaan het spectrum en het sterkste signaal komt bijna tot boven. Zwakke signalen zijn als piekjes te zien en als lichtere strepen in de waterval.
3. Draai aan de VFO of wissel van band. Verwacht: de waterval schuift mee of begint opnieuw, zonder oude strepen op de verkeerde plaats.
4. Meld: is de waterval nog te donker of te fel, wil je een andere hoogte (Instellingen, Weergave: klein, middel, groot), en klopt de plaats van de labels met de pieken.

## A55. Notch en het gat in het geluid (0.18.4)

1. Kijk op de radio of "MN" of "AN" op het scherm staat. Noteer wat er stond.
2. Staat de notch aan: kijk in CWLD in de kaart Audio. Verwacht: binnen enkele seconden de oranje melding "Een gat in het geluid rond ... Hz".
3. Druk op NOTCH tot de aanduiding weg is. Verwacht: de melding in CWLD verdwijnt binnen enkele seconden.
4. Maak één opname van een CW-station met de notch uit en stuur die. Meld ook of CWLD nu beter leest.
5. Stond er geen notch aan en komt de melding toch: meld welk filter (FIL1, 2 of 3), welke bandbreedte en of Twin PBT verdraaid is.

## A56. ADIF exporteren (0.18.5)

1. Open het exportvenster en klik meteen op **Exporteren**, zonder eerst op Controleren te klikken. Verwacht: de controle loopt vanzelf. Zijn er fouten, dan staan ze in het venster. Anders: "N QSO's opgeslagen in ..." met het volledige pad.
2. Klik op **Map openen**. Verwacht: de Verkenner opent de map CWLD in Documenten, met het .adi-bestand erin.
3. Kies een selectie zonder QSO's (bijvoorbeeld "Sinds laatste export" direct na een export). Verwacht: de melding dat er geen QSO's in de selectie zitten, en geen leeg bestand.
4. Lukt het nog steeds niet: stuur een schermafdruk van het exportvenster met de melding.

## A57. Notch en Twin PBT (0.19.0)

1. Verbind de radio. Zet op de radio de handmatige notch aan (NOTCH tot "MN" op het scherm staat). Verwacht: binnen een paar seconden in de kaart Audio "Op de radio: handmatige notch (MN) aan" met de knop **Notch uit**.
2. Klik op **Notch uit**. Verwacht: "MN" verdwijnt van het scherm van de radio en de melding in CWLD gaat weg.
3. Draai aan de binnenste PBT-knop. Verwacht: "Twin PBT verschoven" en de knop **PBT in het midden**. Klik erop. Verwacht: de PBT-aanduiding op de radio staat weer in het midden.
4. Doe hetzelfde met de automatische notch (AN).
5. Komt er geen melding, of doet een knop niets: meld welke stap, dan kloppen de CI-V-commando's voor de MK II niet en zoek ik verder.

## A58. Logboekscherm (0.19.0)

1. Klik bovenaan op **Logboek**. Verwacht: alle QSO's in een tabel, nieuwste eerst.
2. Typ een stuk van een roepletter in het zoekvak. Verwacht: de tabel volgt meteen. Kies ook een band en een datum.
3. Klik op **Kolommen** en zet "Vermogen" aan. Sluit CWLD en open het logboek opnieuw. Verwacht: de kolom staat er nog.
4. Klik op "Deze N QSO's exporteren". Verwacht: het pad en **Map openen**. Lees het bestand in Log4OM in en tel of het aantal klopt.
5. Klik op een rij, wijzig de naam en bewaar. Verwacht: de tabel toont de nieuwe naam.

## A59. Alle velden verbeteren (0.19.1)

1. Open het logboek, klik op een rij. Verwacht: een venster met onder meer datum, tijd (UTC), kHz, band (grijs), mode en opmerking.
2. Wijzig alleen de opmerking en klik **Opslaan**. Verwacht: meteen bewaard, de tabel toont de wijziging.
3. Wijzig de frequentie van een 40m-QSO naar 14025 en klik **Opslaan**. Verwacht: eerst de vraag "Frequentie wijzigen?" met "band verandert van 40m naar 20m". Klik **Terug**: niets is bewaard. Probeer opnieuw en klik **Ja, wijzigen**. Verwacht: de rij toont 20m.
4. Typ bij de tijd "25:00". Verwacht: een foutmelding en niets bewaard.
5. Klap **Mijn station** open, wijzig het vermogen en bewaar. Verwacht: de kolom Vermogen (Kolommen) toont de nieuwe waarde.

## A60. Logboek als CSV (0.19.2)

1. Open het logboek, kies een band en klik op **CSV**. Verwacht: een melding met het pad en **Map openen**.
2. Klik op **Map openen** en dubbelklik het bestand `cwld-logbook-...csv`. Verwacht: Excel toont de kolommen netjes verdeeld, met de koppen in het Nederlands en de namen met accenten juist.
3. Zet via **Kolommen** "Vermogen" aan, exporteer opnieuw. Verwacht: de extra kolom staat in het bestand.
4. Sorteer op roepletters en exporteer opnieuw. Verwacht: dezelfde volgorde als in het scherm.
5. Controleer dat de QSO's niet als "ADIF" geëxporteerd staan na een CSV-export.

## A61. Scrollen en tabbladen (0.19.3)

1. Laat een gesprek met veel tekst lopen. Scrol met het muiswiel een stuk omhoog en wacht tot er nieuwe tekst bijkomt. Verwacht: het beeld blijft staan. Scrol weer helemaal naar onder. Verwacht: nieuwe tekst loopt weer mee.
2. Klik op een ander tabblad en terug. Verwacht: je ziet het einde van het gesprek.
3. Open de kaart Audio en draai aan de notch op de radio tot "MN" verschijnt. Verwacht: de oranje melding komt over het spectrum, de regelaars daaronder blijven op dezelfde plaats.
4. Klik op **Instellingen**, zoek de tabbladen en zet "Maximaal aantal open tabbladen" op 3. Open meerdere gesprekken. Verwacht: de oudste rustige tabbladen sluiten, er verschijnt geen horizontale balk, en een gesprek waarin je al gezonden hebt zonder te loggen blijft open.


## A62. Terugspoelen (0.20.0)

1. Start het luisteren en laat een QSO een halve minuut lopen. Open in de kaart Audio **Terugspoelen** (klik op het driehoekje ervoor). Kies **20 s** en klik **Zelfde toon**. Verwacht: na ongeveer een seconde de tekst van de laatste 20 seconden, met toon en snelheid erboven. De live tekst loopt intussen gewoon door.
2. Zet een tweede station in de doorlaat (draai de VFO tot er twee tonen hoorbaar zijn). Klik **Sterkste**. Verwacht: onder "Gehoord" staan beide tonen. Klik op de andere toon. Verwacht: de tekst van het andere station.
3. Klik op een roepletter in het resultaat. Verwacht: hij komt in het QSO-paneel, zoals bij een klik in het gesprek.
4. Klik **Laatste minuut bewaren (WAV)**. Verwacht: een melding met het pad. Open de opnamemap (zelfde map als de gewone opnames) en dubbelklik het bestand. Verwacht: een minuut geluid, met de frequentie in de naam.
5. Stop het luisteren. Verwacht: Terugspoelen verdwijnt uit de kaart.

## A63. Steil filter (0.20.0)

1. Klik in de kaart Audio op **Afstellen** en daarna op **Steil**. Verwacht: Steil licht op, Smal en Breed niet. De rode lijnen in de audio-waterval blijven even breed als bij Smal.
2. Zoek een zwak station met een sterk station 60 tot 150 Hz ernaast (een contestband is ideaal). Vergelijk Smal en Steil een minuut elk. Noteer welk de minste rommel geeft.
3. Op een rustige plek zonder signaal: laat Steil een minuut aanstaan. Noteer of er tekens uit de ruis komen. Doe hetzelfde met Smal.
4. Klik **Breed** en daarna terug **Steil**. Klik op **Instellingen** en kies links **Decoder**. Verwacht: dezelfde keuze staat daar, ook na een herstart van CWLD.

## A64. Gespelde letters (0.20.0)

1. Laat iemand (of een tweede programma, of de oefenmodus) `NAME IS H A N S` zenden met lange pauzes tussen de letters. Verwacht: CWLD toont `HANS`. De letters verschijnen eerst als voorlopige tekst en worden samen definitief zodra het volgende woord of een pauze van enkele seconden komt.
2. Zo ook `DE D L 4 S A V K`. Verwacht: `DL4SAV K`.
3. Een normaal einde `73 E E` of `R R TU`. Verwacht: blijft zoals het is.
4. Klik op **Instellingen**, kies links **Decoder** en zet "Gespelde letters samenvoegen" uit. Herhaal stap 1. Verwacht: `H A N S`.

## A65. Teksten van chips en toetsen (0.20.1)

1. Klik met de rechtermuisknop op de chip **CQ**. Verwacht: het venster "Teksten van chips en toetsen" opent op CQ. Verander de tekst in `CQ CQ DE {MYCALL} K` en klik **Bewaren**. Verwacht: in de lijst staat "aangepast" bij CQ.
2. Klik **Live**, bevestig, en klik op de chip CQ. Verwacht: de radio zendt de nieuwe tekst.
3. Open het venster opnieuw (rechtsklik op CQ) en klik **Standaard herstellen**. Verwacht: de oude tekst staat terug.
4. Typ in een tekst `{FOO}` en een `!`. Verwacht: twee oranje waarschuwingen. Klik **Ongedaan maken**.
5. Klik **+ Eigen chip**, geef de naam `WX` en de tekst `WX HR SUNNY`, bewaar. Verwacht: een chip WX met een stippelrand achteraan. Zet hem met de pijl een plaats naar voren. Verwacht: hij schuift ook op het hoofdscherm. Verwijder hem daarna met **Chip verwijderen** en **Ja**.
6. Zet contest mode aan, kies het profiel **CQ WW DX Contest CW** en klik rechts op **F3**. Pas de tekst aan naar `TU {MYCALL} TEST`, bewaar, en kies daarna in het venster het profiel **CQ WPX**. Verwacht: daar staat F3 nog op de standaardtekst.
7. Kies het profiel **UBA DX Contest CW**. Wissel boven de toetsen tussen de twee uitwisselingen en klik telkens rechts op **F2**. Verwacht: twee verschillende teksten.
8. Verander een tekst en druk **Esc** zonder te bewaren. Verwacht: de vraag of je wilt bewaren. Sluit de app en start opnieuw. Verwacht: bewaarde teksten zijn er nog.

## A66. CQ herhalen (0.20.1)

Veiligheid: laag vermogen of een dummyload. CQ herhalen zendt alleen zolang je aan het station zit.

1. Klik **Live** en daarna **CQ herhalen (Alt+R)**. Verwacht: de knop wordt oranje, de regel "CQ wordt geroepen (1 van 10)". Na de CQ: "Luisteren. Opnieuw CQ over 6 s", aftellend, daarna een nieuwe CQ.
2. Druk tijdens de pauze op een willekeurige toets (bijvoorbeeld de spatiebalk in het tekstvak). Verwacht: CQ herhalen stopt, met de reden onder het zendvak.
3. Start opnieuw en sein tijdens de pauze kort met de paddle. Verwacht: stopt met "de paddle seinde".
4. Start opnieuw en laat een ander station (of een tweede ontvanger) tijdens de pauze antwoorden. Verwacht: stopt met "er lijkt een station te antwoorden". Noteer of ruis het ook soms stopt.
5. Start opnieuw en draai aan de VFO. Verwacht: stopt. Start opnieuw en minimaliseer CWLD. Verwacht: stopt.
6. Zet in het tekstvenster onder "CQ herhalen en de paddle" de pauze op 3 s en het maximum op 2. Start. Verwacht: na twee CQ's stopt het vanzelf.
7. Contest mode, Run: Alt+R roept de F1-tekst. Schakel naar S&P (Ctrl+R). Verwacht: CQ herhalen stopt en de knop is grijs.
8. Controleer op de radio of Alt+R niets anders doet in Windows of op het toetsenbord (botsing met een andere sneltoets).

## A67. Getypte tekst wacht op de paddle (0.20.1)

1. Klik **Live**. Begin met de paddle een lange zin te seinen en typ intussen `TNX FER CALL` in het zendvak en druk Enter. Verwacht: de tekst staat onder "Wacht op de paddle" en de radio zendt nog niets.
2. Stop met de paddle. Verwacht: ongeveer een seconde later zendt de radio de getypte tekst, en ze verschijnt in het gesprek.
3. Herhaal, maar druk **Esc** terwijl de tekst wacht. Verwacht: de wachtende tekst verdwijnt, er wordt niets gezonden.
4. Klik tijdens het seinen met de paddle op een chip. Verwacht: ook die wacht.
5. Test met semi break-in en met full break-in (QSK) op de radio. Noteer of de tekst ooit midden in je paddle-zin start (dan is de stilte te kort).
6. Zet in het tekstvenster "Getypte tekst wacht zolang de paddle zendt" uit en herhaal stap 1. Noteer wat de radio doet als beide tegelijk komen.

