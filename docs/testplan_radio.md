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
