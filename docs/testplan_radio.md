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
