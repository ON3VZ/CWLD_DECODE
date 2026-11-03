# CWLD: ontwerp en architectuur (stap 2b)

Datum: 3 november 2026. Versie 6. Bouwt voort op `bevindingen.md`, `bevindingen_qmx.md` en `CWLD_ontwerprichtingen.html`.

**Wijzigingen in versie 6 (SWR-bewaking):** (19) De SWR wordt van de radio gelezen terwijl hij zendt (Icom `15 12`, elke 100 ms in een snelle meetlus naast de gewone 500 ms). (20) Na **twee metingen na elkaar boven de grens** (ongeveer 0,2 seconden) stopt CWLD de tekst (`FF`), toont een **rood alarm** en blokkeert zenden tot de operator bevestigt dat de antenne gecontroleerd is. Eén meting is niet genoeg, tegen een stoorpiek bij het indrukken van de sleutel. (21) Instellingen: bewaking aan of uit (standaard **aan**), grens (standaard **1,5**, begrensd tot 1,1 en 3,0, ook als iemand het bestand met de hand aanpast), QRX na het stoppen (standaard **uit**, zie hieronder). (22) **QRX als laatste zending**: na het stoppen van de lopende tekst zendt CWLD eenmalig `QRX`, zodat het andere station weet dat je weggaat. Dit staat **standaard aan** op uitdrukkelijke wens van de operator, en is uitschakelbaar omdat het nog een keer zendt in een antenne met hoge SWR. Dat werkt alleen in LIVE. Of de Icom de tekst na `FF` meteen aanvaardt is een **[TEST]**. (23) Een paddle-zending kan de app niet stoppen. Het alarm wordt wel getoond. (24) De QMX geeft de SWR volgens de CAT-handleiding van QRP Labs (door de operator aangehaald, nog niet zelf gelezen) via `SW;`, antwoord `SWnnn;` in honderdsten (121 is 1,21). CWLD vraagt dit elke 100 ms alleen tijdens zenden. Een kaal `SW;` (ontvangst) of een waarde onder 1,00 is geen meting. Het diagnosescherm in een terminalprogramma is voor mensen en wordt niet gebruikt **[VERIFIEER]**. (25) De bewaking is een **tweede lijn** naast de eigen bescherming van de radio, geen vervanging. De SWR-schaal van de Icom (0 is 1,0, 48 is 1,5, 80 is 2,0, 120 is 3,0) is uit de IC-7300-gegevens en moet voor de MK II bevestigd worden **[TEST]**.

**Wijzigingen in versie 5 (fase 2, gebouwd):** (11) Icom-driver: tekst in blokken van maximaal 30 tekens op woordgrenzen, een spatie achteraan elk blok behalve het laatste, **één blok tegelijk**. Het volgende blok vroeger sturen (`pipeline`) staat **standaard uit** en schakelt zichzelf uit als de radio een tweede blok weigert (NG). (12) QMX-driver: tekst **per woord** met een spatie erachter, alleen toegevoegd als het zeker in de `KY`-buffer past (leeg: 80 tekens, onder 75%: hoogstens 20, boven 75%: niets, onbekend: niets). (13) De QMX leest bij een `IF`-bericht alleen de frequentie. Modus en TX komen uit `MD;` en `TQ;`, want de rest van `IF` is niet gecontroleerd. De S-meter van de QMX wordt nog niet gelezen (`SM;` is onduidelijk). (14) Stop op de QMX: wachtrij leeg en een bericht van 24 spaties **[TEST]**. (15) Verboden commando's (`PS...`, `KS00`) worden in de QMX-driver zelf geweigerd, ook als iemand ze via `Raw` probeert. (16) Een radio die niet verbonden is toont niets (`NullRig`), nooit nepgegevens. (17) Maximum 60 seconden per bericht. (18) Het testplan voor jouw radio's staat in `testplan_radio.md`.

**Wijzigingen in versie 4:** (8) Paddle en toetsenbord door elkaar in één gesprek, met aparte bubbelsoort "Jij (paddle)" (2.8, 3.5). (9) De gedecodeerde roepletters vullen automatisch de variabele `{CALL}` (en waar gevonden `{NAME}` en `{QTH}`), tenzij de gebruiker ze vastzet (2.3, 2.8). (10) Variabelen herverdeeld: `{NAME}` en `{QTH}` zijn van het andere station, `{MYNAME}` en `{MYQTH}` van jou.

**Wijzigingen in versie 3:** (4) Eigen logboek met ADIF-export, ook voor WWFF/ONFF-activaties (3.11, scherm Logboek in 2.2). (5) QMX-audio: zijtoon via USB bevestigd (zie `bevindingen_qmx.md`). (6) Thuislocator gecorrigeerd naar JO20EE. (7) Modi LIVE, LUISTEREN en OEFENEN met een zendslot in de kern (2.7, 3.5).

**Wijzigingen in versie 2:** (1) Log4OM-koppeling herzien, WSJT-X-formaat is niet langer de standaard (3.8). (2) Nieuw woordenboek met Q-codes en afkortingen, bij lezen en bij zenden (2.6 en bijlage A). (3) De QRP Labs QMX als tweede radio, met een tweede stuurprogramma en capabilities per radio (3.4, 3.5).
Alles met **[VERIFIEER]** is uit geheugen of afleiding en moet vóór het bouwen tegen de bron nagekeken worden. Alles met **[TEST]** moet op Kristofs radio gemeten worden (zie bevindingen paragraaf 8).

---

## 1. Genomen beslissingen (aanvulling op hoofdstuk 3 van de projectinstructie)

| Onderwerp | Beslissing |
|---|---|
| Uiterlijk | **Richting A (Gesprek) als basis.** Uit richting C komt de **signalenlijst als uitklapbaar zijpaneel**. Richting B (grote frequentie, ondertitels) komt terug als **"Groot scherm"-weergave** voor contests, later. |
| Webframework | **Svelte 5** (runes) met TypeScript, gebundeld met Vite |
| Open beslissing 1 (Log4OM) | **Loggen via de ADIF MESSAGE-service van Log4OM** (geen FT8-formaat). Live velden via JT-status of eigen CAT van Log4OM zijn opties. Zie 3.8. |
| Open beslissing 2 (paddle) | Eerst testen of de USB-audio het eigen zendsignaal bevat. Zo niet: alleen een TX-indicator |
| Open beslissing 3 (signatuur) | Eerste testreleases ongetekend, daarna SignPath Foundation aanvragen |
| Open beslissing 4 | Svelte, zie boven |
| Radio's | **IC-7300MK2 eerst, QRP Labs QMX als tweede** (Kenwood TS-480-achtig CAT). Capabilities per radio. |
| Woordenboek | Ingebouwd, tweetalig, voor Q-codes, afkortingen en prosigns. Tonen bij lezen, kiezen bij zenden. |
| Modi | **LIVE, LUISTEREN, OEFENEN.** De app start nooit in LIVE. Buiten LIVE blokkeert een zendslot in de kern alle zendcommando's. Zie 2.7 en 3.5. |
| Logboek | **Eigen logboek in de app** (SQLite) met stationsprofielen (thuis, portable), activaties en **ADIF-export** met WWFF-velden. Log4OM blijft optioneel. Zie 3.11. |

---

## 2. Gebruikersinterface

### 2.1 Hoofdscherm (richting A + zijpaneel uit C)

```
+----------------------------------------------------------------------------------+
| o IC-7300MK2   7.025.40 MHz   CW  FIL2   22 WPM   S-meter   [Signals]  Log4OM o  |  statusbalk
+----------------------------------------------------------------------------------+
| waterfall-strook (84 px), pitch-lijn, labels van gedetecteerde signalen          |
+---------------------------------------------------------+------------------------+
|                                                         | DIT QSO                |
|   gesprek (bubbels)                                     |  call, naam, QTH, RST  |
|   ontvangen links, eigen tekst rechts                   |  badges (nieuw land)   |
|   live bubbel met vervaagde onzekere tekens             | SUGGESTIES decoder     |
|                                                         |  [Naam: Hans] [QTH..]  |
|                                                         | [Log naar Log4OM]      |
+---------------------------------------------------------+------------------------+
| chips: [CQ F1] [Antwoord DK1AB F2] [RST+naam F3] [73 F4] [AGN? F5] [QRZ? F6] [+]  |
| [ tekstvak ..................................... ] snelheid ---o--- [Zend] [Stop] |
+----------------------------------------------------------------------------------+
```

Zijpaneel **Signalen** (uit C): schuift van links in over de waterfall en het gesprek, sneltoets `S`. Per signaal een kaart met roepletters (of "?"), offset in Hz, WPM, sterkte-balkje en de laatste tekst. Klik op een kaart: de decoder-focus en de pitch-lijn springen naar dat signaal, het gesprek toont dat station. Standaard **dicht**, zodat de eerste indruk eenvoudig blijft.

### 2.2 Weergaven en schermen

| Scherm | Doel |
|---|---|
| **Hoofdscherm** | Zie 2.1 |
| **Groot scherm** (later, uit B) | Grote frequentie en ondertitels voor contests of op afstand lezen |
| **Chips bewerken** | Lijst met chips: naam, tekst, sneltoets, volgorde slepen |
| **Instellingen** | Secties: Radio, Audio, Station, Decoder, Zenden, Log4OM, Uiterlijk en taal, Updates |
| **Begeleide eerste start** | 4 stappen, zie 2.5 |
| **Logboek** | Tabel met QSO's, filters, bewerken, activaties en exporteren (3.11) |
| **Diagnose** | Ruwe CI-V-berichten, UDP-berichten, audio-niveau. Voor hulp bij problemen en voor de [TEST]-metingen |

### 2.3 Gedrag en regels

- **Gesprek**: bubbels per station. Een nieuw station of een nieuwe frequentie begint een nieuw gesprek (met scheidingslijn). Het gesprek wordt in het geheugen en in een lokaal dagbestand bewaard (JSON-lijnen), niet in de cloud.
- **Onzekere tekens**: de decoder levert per teken een waarschijnlijkheid. Onder 90% staat het teken vervaagd. Muisaanwijzer toont de alternatieven (bv. "N 82%, 9 17%"). Na afloop van een bubbel wordt de beste lezing bevroren. Klik op een vervaagd teken om het te corrigeren, dat verbetert ook het QSO-paneel.
- **Roepletters**: gele tekst met streepjeslijn. Een gedecodeerde roepletterreeks komt **vanzelf** in het QSO-paneel en dus in `{CALL}` (zie 2.8). Klik zet ze handmatig vast.
- **RST**: groen. Wordt voorgesteld in het QSO-paneel als "RST rcvd".
- **Chips**: variabelen `{CALL}`, `{MYCALL}`, `{NAME}`, `{RST}`, `{QTH}`, `{MYNAME}`, `{MYQTH}`, `{TIME}`, `{SERIAL}`. Is een variabele leeg, dan wordt de chip **niet verzonden** en kleurt de ontbrekende variabele rood (geen "DE  KN" op de lucht). Een klik zet de tekst **in het tekstvak** of verstuurt meteen, naar keuze per chip ("Direct zenden" aan of uit). Standaard staat het op **in het tekstvak**, dus de gebruiker ziet eerst wat er gaat uitgaan.
- **Prosigns**: in het tekstvak als `<AR>`, `<SK>`, `<BT>`, `<KN>`. Vertaling naar CI-V `^AR` enz. is een **[TEST]**.
- **Noodstop**: knop "Stop" en toets `Esc`. Voert uit: CI-V 0x17 met data `FF`, wist de eigen wachtrij, zet de pad-status op "Gestopt". Werkt altijd, ook als het tekstvak de focus heeft.
- **Paddle**: werkt altijd en wordt nooit geblokkeerd. Tijdens het zenden van de radio ziet de app via TX-status (`1C 00`) dat er gezonden wordt en toont "TX" in de statusbalk. De eigen zendwachtrij pauzeert zolang de radio al zendt.
- **Waarschuwingen** (banner boven het gesprek, geen modale vensters): radio niet verbonden, Break-in staat uit (0x17 zendt dan niet), mode is niet CW, geen audio-ingang, Log4OM-poort niet bereikbaar.
- **Toegankelijkheid**: alles bedienbaar met toetsenbord, minimale tekstgrootte 15 px, contrast minstens 4,5 op 1, kleur nooit het enige signaal (roepletters ook onderstreept).

### 2.4 Instellingen met uitleg

Elke instelling heeft een korte uitleg in gewone taal en een knop "Standaard herstellen". Voorbeeld:

> **CI-V Transceive** (aanbevolen: aan)
> Als dit aan staat, meldt de radio zelf elke frequentiewijziging. CWLD volgt dan direct de draaiknop.
> *Zo vind je het op de radio:* tik op MENU, dan SET, dan Connectors, dan CI-V, en zet "CI-V Transceive" op ON.

Dit "hoe vind je het op de radio"-blokje komt bij **elke** radio-instelling (afspraak uit de projectinstructie: altijd ook uitleggen hoe).

### 2.5 Begeleide eerste start (4 stappen)

1. **Radio vinden.** CWLD zoekt seriële poorten met naam "IC-7300MK2 Serial Port A (CI-V)" (Windows: via de poortbeschrijving, Linux: via `/dev/serial/by-id`). Eén gevonden: automatisch kiezen. Niets gevonden: uitleg met drie controles (kabel, radio aan, Icom-driver geïnstalleerd) en een knop "Opnieuw zoeken". Daarna een test: CWLD vraagt de frequentie op en toont ze.
2. **Radio-instellingen controleren.** CWLD leest wat het kan lezen (CI-V Transceive, Break-in, mode) en toont groene of rode vinkjes met de "hoe vind je het op de radio"-uitleg.
3. **Audio kiezen.** Ingang "USB Audio CODEC" voorgesteld, met niveaumeter en uitleg over Windows-geluidsniveaus (project-les uit MRP40: geen audioverbeteringen, niveau rond 70 tot 73 [TEST]). Bij een QMX komt er uitleg bij over de menu-instellingen **Audio, Sidetone volume** en **Audio, Sidetone abs/rel** (zie `bevindingen_qmx.md`, paragraaf 7), met de stappen waar je die in het QMX-menu vindt.
4. **Station.** Roepletters (ON3VZ, ook ON3KC, ON3KRB als lijst), naam, QTH, locator, Log4OM-ADIF-poort (voorstel 2333, de eerste start toont uitleg waar je die in Log4OM vindt). Slot: korte test met een **stille test**: de app toont wat ze zou zenden, zonder te zenden.

### 2.6 Q-codes en afkortingen (woordenboek)

Eén ingebouwd **woordenboek** in een gegevensbestand (`dictionary.json`, Engels en Nederlands, geen vaste tekst in de code) met Q-codes, CW-afkortingen, prosigns en rapporten. De startinhoud staat in bijlage A. Het woordenboek wordt op drie plaatsen gebruikt.

**A. Lezen: betekenis bij ontvangen tekst**

Instelling "Afkortingen tonen" met vier standen:

1. **Uit.**
2. **Bij aanwijzen** (tooltip): wijs op `QRS` en je ziet "Zend langzamer" of, bij `QRS?`, "Moet ik langzamer seinen?".
3. **Onder de bubbel**: een kleine grijze regel met de betekenis van de hele zin. Voorbeeld: `UR RST 599 QTH HOBOKEN` geeft "Jouw rapport is 599. Mijn locatie is Hoboken."
4. **Ter plaatse vervangen**: `QRS` wordt "zend langzamer" (zoals "expanded Q codes" in MRP40), de oorspronkelijke tekst blijft zichtbaar bij aanwijzen.

Regels:
- Een Q-code **met vraagteken** wordt als vraag uitgelegd, **zonder vraagteken** als antwoord of mededeling. Dat is de officiële betekenis: `QRL?` is "Bent u bezet?", `QRL` is "Ik ben bezet".
- Waarden achter een code worden herkend: `QRS 15`, `RST 599`, `5NN` (cut number N = 9), `PWR 5W`. De uitleg noemt de waarde.
- **Onzekere tekens** (decoder onder 90%) worden niet vertaald, zodat er geen valse uitleg verschijnt.
- Taal van de uitleg: gelijk aan de app-taal, of apart instelbaar (bv. Nederlandse app met Engelse uitleg).

**B. Zenden: afkortingen kiezen**

- **Aanvullen tijdens typen**: typ `QR` en een lijstje toont QRA, QRG, QRK, QRL, QRM... met de betekenis ernaast. Pijltjestoetsen en `Tab` om in te vullen.
- **Afkortingenpaneel** (knop naast het tekstvak, sneltoets `Ctrl+K`): zoekbaar paneel met tabbladen **Q-codes**, **Afkortingen**, **Prosigns** en **Rapporten**. Een klik zet het item in het tekstvak, de betekenis staat erbij.
- **Voorgedefinieerde waarden**: sommige items hebben een waardekiezer.

| Item | Kiezer |
|---|---|
| `QRS` en `QRQ` | snelheid in WPM (met schuifregelaar, optioneel meteen de keyer-snelheid zetten) |
| `RST` | R (1 tot 5), S (1 tot 9), T (1 tot 9) met snelkeuze 599, 579, 449 en "5NN" |
| `PWR` | 5 W, 10 W, 100 W of eigen waarde |
| `QRG` | frequentie van de actieve VFO |
| `QTH`, `LOC` | `{MYQTH}` en `{MYLOC}` uit de instellingen |
| `WX` en `TEMP` | korte lijst (SUNNY, CLOUDY, RAIN) en een temperatuur |

- **"Wat zeg ik hier?"**: onder het tekstvak staat in kleine tekst de betekenis van wat je typt, in gewone taal. Handig voor beginners en om te controleren dat je geen onzin uitzendt.

**C. Woordenboek beheren**

- Gebruiker kan **eigen afkortingen** toevoegen (bv. clubjargon) en eigen vertalingen, met import en export als JSON.
- Chips (2.3) mogen elk woordenboekitem bevatten. Het aanvullen werkt ook in de chip-editor.

**Gegevensmodel** (schets):

```json
{
  "id": "QRS", "kind": "q", "text": "QRS",
  "question": { "en": "Shall I send more slowly?", "nl": "Moet ik langzamer seinen?" },
  "statement": { "en": "Send more slowly.", "nl": "Sein langzamer." },
  "param": { "type": "wpm", "min": 5, "max": 40, "default": 15 },
  "tags": ["speed"]
}
{ "id": "AGN", "kind": "abbr", "text": "AGN", "en": "again", "nl": "opnieuw" }
{ "id": "AR", "kind": "prosign", "text": "AR", "morse": ".-.-.", "en": "end of message", "nl": "einde bericht" }
```

### 2.7 Modi: Live, Luisteren en Oefenen (nieuw in versie 3)

Idee van Kristof: de app kan decoderen zonder te zenden, bijvoorbeeld met de radio op SSB, en zo kun je er ook mee oefenen. Uitwerking: drie modi met een schakelaar in de statusbalk, altijd zichtbaar, elk met een eigen kleur en label.

| Modus | Mag CWLD naar de radio zenden? | Audiobron | Waarvoor |
|---|---|---|---|
| **LIVE** | Ja, alleen na een bewuste klik of toets | radio | normaal gebruik |
| **LUISTEREN** | **Nee, nooit** | radio, in elke radiomodus (ook SSB) | meeluisteren, decoder testen, afstemmen, opnames voor de testbank maken |
| **OEFENEN** | **Nee, nooit** | **bestand of generator**, geen radio nodig | typen en chips oefenen, kopieeroefening, demonstratie, ontwikkelen zonder radio |

**Luisteren**
- De radio mag op SSB, CW of een andere modus staan. De decoder werkt op de audio en dus ook als de radio op USB of LSB staat. De CW-toon zit dan op een toonhoogte die afhangt van waar je afstemt, en het filter is breder (bijvoorbeeld 2,4 kHz). De waterfall en de pitch-lijn tonen waar het signaal zit. "Modus is niet CW" is hier een gewone info-melding, geen waarschuwing.
- Zendknoppen en chips zijn grijs, typen mag als concept, verzenden niet. Een duidelijk label "LUISTEREN" staat in de statusbalk.
- Frequentie volgen en afstemmen via CWLD werken gewoon, want dat zendt niet.

**Oefenen**
- **Stille zending**: de tekst wordt verwerkt zoals bij echt zenden (opschonen, knippen, timing, wachtrij), maar **niets gaat naar de radio**. CWLD speelt de zijtoon af via de pc-luidsprekers, met instelbare toonhoogte en snelheid.
- **Bron voor de decoder**: gegenereerde CW (de generator uit de testbank), een WAV-bestand of een ingebouwde reeks oefen-QSO's. De decoder werkt er op alsof het van de radio komt.
- **Kopieeroefening**: je hoort CW zonder tekst, typt wat je hoort, en de app toont je tekenfouten en snelheid. De juiste tekst is bekend omdat de app ze zelf gemaakt heeft.
- **Virtuele gesprekspartner (latere uitbreiding)**: een gesimuleerd station antwoordt op wat jij typt, met een zijtoon. Zo oefen je een volledig QSO (CQ, antwoord, rapporten, 73) met de chips en het woordenboek, zonder te zenden.

**Opnemen voor de testbank**: knop "Bewaar laatste minuut" in LUISTEREN. Dat geeft een WAV-bestand (originele samples, 48 kHz) plus een tekstbestand met de gedecodeerde tekst, frequentie, modus, snelheid en een notitieveld "wat werd er echt gezonden". Zo maak je de opnames voor stap 3 zelf, zonder risico.

**Schakelen naar LIVE**
- Altijd een bewuste klik, bij de eerste keer per sessie met een bevestiging die vermogen en modus toont.
- CWLD **start nooit automatisch in LIVE**.
- CWLD kan de radio zelf niet blokkeren: de paddle en de PTT van de radio werken altijd (zoals afgesproken). LUISTEREN en OEFENEN beperken alleen wat CWLD doet.

---

### 2.8 Paddle en toetsenbord door elkaar, en automatische {CALL} (nieuw in versie 4)

**Eén gesprek, drie bronnen van jouw tekst.** Wat je zelf zendt staat altijd in dezelfde reeks bubbels, in de volgorde waarin het gezonden werd, ongeacht hoe je het zendt:

| Bron | Bubbel | Hoe de app het weet | Betrouwbaarheid |
|---|---|---|---|
| Toetsenbord of chip | "Jij", effen blauw | De app kent de tekst zelf | Exact |
| Paddle op de radio | "Jij (paddle)", blauw met streeplijn | Gedecodeerd uit wat de radio uitzendt | Als een decodering: onzekere tekens vervaagd, zoals bij ontvangen tekst |
| Andere zender | Grijs, met roepletters | Decoder | Als een decodering |

**Hoe de paddle zichtbaar wordt** (per radio, via `capabilities`):

- **QMX**: de eigen decoder van de radio levert tekst via `TB` en de zijtoon komt over USB. Dit is de makkelijkste weg. Zie `bevindingen_qmx.md`.
- **IC-7300 MK II**: er is geen CI-V-commando bekend dat de gesleutelde tekens als tekst doorgeeft **[VERIFIEER]**. De route is de USB-audio decoderen, op voorwaarde dat de meeluistertoon daarin zit **[TEST]**. Valt dat tegen, dan blijft over: de TX-status (`1C 00`) tonen als "paddle zendt" zonder tekst.
- **Geen dubbele weergave**: zendt de app zelf tekst, dan wordt de decodering van dezelfde toon onderdrukt. Alleen wat niet uit de app komt, telt als paddle.
- **Gemengd gebruik**: begin je met de paddle terwijl er tekst uit de app loopt, dan toont de app dat de wachtrij onderbroken of in pauze is. Wat de radio doet (onderbreekt de paddle de keyer-wachtrij of wacht ze?) is een **[TEST]**. De radio-keyer wordt nooit geblokkeerd.
- **LUISTEREN en OEFENEN**: de modus van de app regelt alleen wat de app zelf zendt. De paddle op de radio zendt altijd, ook als de app op LUISTEREN staat, want dat gebeurt in de radio. Het gesprek toont die paddle-tekst gewoon, en de banner meldt dat de radio zendt terwijl de app op LUISTEREN staat.

**Automatische {CALL}.** Zodra de decoder een bericht van een ander station afwerkt, bepaalt de app:

1. **Afzender**: de roepletters na `DE`. Zonder `DE`: de eerste roepletterreeks die niet van jou is. Je eigen roepletters (instellingen) worden nooit genomen.
2. **Naam en QTH**: het woord na `NAME` (of `OP`) en na `QTH`. Ontbreekt het sleutelwoord, dan wordt niets geraden.
3. **Invullen**: `{CALL}` krijgt de afzender. Is dat een nieuw station, dan worden naam en QTH leeg gemaakt en met de nieuwe gegevens gevuld. Naam en QTH vullen alleen lege velden, zodat jouw correcties blijven staan.
4. **Vastzetten**: typ of klik je zelf een roepletterreeks, dan staat het veld op "Door jou vastgezet" en overschrijft de decoder het niet meer. Een knop "Weer de decoder volgen" geeft het terug aan de decoder.
5. **Onzeker**: een roepletterreeks met lage zekerheid uit de decoder wordt als voorstel getoond (vervaagd, met de alternatieven) en pas ingevuld in `{CALL}` na bevestiging of zodra ze boven de drempel komt. Chips met `{CALL}` blijven tot dan geblokkeerd, in lijn met "een lege variabele wordt nooit gezonden". Het doel is nooit een fout roepletter op de lucht.
6. **Eerste versie in fase 1**: de afleiding werkt op tekst uit de demo. De drempel voor onzekere tekens volgt in fase 3 met de echte decoder.

**Variabelen, herverdeeld:** `{CALL}`, `{NAME}`, `{QTH}`, `{RST}` horen bij het **andere** station. `{MYCALL}`, `{MYNAME}`, `{MYQTH}` zijn jouw eigen gegevens uit de instellingen. De standaardchip "Rapport" is dus `{CALL} DE {MYCALL} UR RST {RST} {RST} NAME {MYNAME} QTH {MYQTH} HW? K`.

---

## 3. Architectuur

### 3.1 Overzicht

```
                 +--------------------- Tauri 2 proces ----------------------+
                 |                                                           |
 Svelte 5 UI <-- events/channels --> cwld-app (commando's, state, instellingen)
                 |                      |             |            |         |
                 |                 cwld-radio     cwld-audio   cwld-udp     |
                 |                 (CI-V, rig     (cpal in,    (ADIF/UDP    |
                 |                  trait)         ring buf)    formaat)     |
                 |                      |             |                      |
                 |                  cwld-keyer     cwld-dsp --> cwld-decoder |
                 |                  (wachtrij,     (FFT, filter,  (HMM, beam, |
                 |                   chunks, stop)  detectie)      taalmodel) |
                 +-----------------------------------------------------------+
                       |                      |
                  seriële poort A         USB-audio
                  (IC-7300MK2)             (IC-7300MK2)
```

### 3.2 Workspace-indeling (Rust)

```
cwld/
  Cargo.toml                 workspace
  crates/
    cwld-types/              gedeelde types (Event, Command, Hypothesis, QsoState)
    cwld-radio/              trait Rig, Icom CI-V stuurprogramma, simulator
    cwld-keyer/              zendwachtrij, chunking, timingschatting, noodstop
    cwld-audio/              cpal-ingang, resampling, ring buffer
    cwld-dsp/                bandfilter, AFC, envelope, signaaldetectie, waterfall-FFT
    cwld-decoder/            timingmodel, beam search, taalmodel, signaalmanager
    cwld-udp/                ADIF-berichten, optioneel JT-status, UDP-verzending
    cwld-log/                logboek (SQLite), stationsprofielen, activaties, ADIF-schrijver en lezer
    cwld-testbench/          generator, meetscript, WAV-ingang (stap 3)
  src-tauri/                 cwld-app: Tauri-commando's, state, instellingen, updater
  ui/                        Svelte 5 + Vite + TypeScript
  i18n/en.json  nl.json
  docs/
```

Regel: **`cwld-radio`, `cwld-dsp` en `cwld-decoder` kennen Tauri niet.** Ze zijn gewone bibliotheken met unit-tests en kunnen door de testbank en de command-line gebruikt worden.

### 3.3 Threads en gegevensstromen

| Thread | Taak | Communicatie |
|---|---|---|
| **Audio-callback** (cpal) | Alleen samples in een ring buffer kopiëren. Geen allocaties, geen locks. | lock-free ring buffer (SPSC) |
| **DSP-thread** | Samples uit de buffer halen, naar 8 kHz brengen, FFT voor de waterfall, per gevolgd signaal band-filteren en envelope maken | stuurt waterfall-rijen en envelopes door |
| **Decoder-thread(s)** | Per gevolgd signaal een decoder. Begin met één decoder voor het gekozen signaal. Later meerdere (richting C). | stuurt `DecodeUpdate` events |
| **Radio-thread** | Eigenaar van seriële poort A. Leest continu bytes, ontleedt CI-V-frames, stuurt `RadioEvent`. Voert een commandowachtrij uit. | kanalen (`crossbeam`) |
| **Keyer-thread** | Houdt de zendwachtrij bij en stuurt blokken naar de radio-thread. Hoort `Esc` met voorrang. | kanaal met prioriteit |
| **UDP-thread** | Verstuurt ADIF-berichten (en optioneel JT-status) naar Log4OM | kanaal |
| **Tauri main + UI** | Commando's van de UI, events naar de UI | Tauri events en channels |

**Waterfall naar de UI**: ruwe bytes (u8 per pixel) via een Tauri `Channel`, 20 rijen per seconde, 512 pixels breed. Geen JSON voor beeldgegevens. Svelte tekent in een `<canvas>` en scrolt zelf.

**Decoder-events naar de UI**: JSON, laag volume (tientallen per seconde), met hypotheses.

### 3.4 Radiolaag

```rust
// cwld-radio (schets, Engels zoals in de code)
pub trait Rig: Send {
    fn info(&self) -> RigInfo;                         // naam, adres, mogelijkheden
    fn connect(&mut self, port: &PortId) -> Result<()>;
    fn poll(&mut self) -> Result<Vec<RigEvent>>;       // niet-blokkerend
    fn set_frequency(&mut self, hz: u64) -> Result<()>;
    fn set_cw_speed(&mut self, wpm: u8) -> Result<()>;
    fn send_cw(&mut self, text: &str) -> Result<()>;   // max 30 tekens, rig knipt zelf niet
    fn stop_cw(&mut self) -> Result<()>;               // Icom: 0x17 met FF
    fn capabilities(&self) -> Capabilities;            // bv. max_cw_chunk = 30, has_transceive
}

pub enum RigEvent {
    Frequency { hz: u64, vfo: Vfo },
    Mode { mode: Mode, filter: u8 },
    Tx(bool),
    SMeter(u16),
    CwSpeed(u8),
    Raw(Vec<u8>),    // voor het diagnosescherm
}
```

- **Icom CI-V stuurprogramma**: frame `FE FE <naar> <van> <cmd> [<sub>] [data] FD`. Adres radio **B6h** (instelbaar), adres pc **E0h**. Het programma moet **echo** van de eigen frames herkennen en negeren (CI-V is eendraads, ook via USB kan echo aan staan, instelling `CI-V USB Echo Back`).
- **Commandotabel apart in een gegevensbestand** (`icom_ic7300mk2.toml`), niet in code, zodat de verschoven IC-7300MK2-commando's (zie bevindingen 2.1) en later andere Icom-modellen gewoon een ander bestand zijn.
- **Capabilities per radio** (de keyer en de UI vragen deze op en kijken niet naar modelnamen):

```rust
pub struct Capabilities {
    pub cw_text: CwTextMode,        // Chunked { max: 30 } (Icom) of Buffered { size: 80 } (QMX)
    pub cw_abort: AbortMethod,      // Icom: FF via 0x17. QMX: nog te meten
    pub radio_decoder_text: bool,   // QMX: TB; Icom: nee
    pub dtr_keying: bool,
    pub pushes_status: bool,        // Icom transceive, QMX AI2
    pub forbidden: &'static [&'static str], // QMX: "KS00", "PS0"
}
```

- **Tweede stuurprogramma: `kenwood_ts480` met een QMX-profiel** (zie `bevindingen_qmx.md`). Tekst-CAT met `;` als einde, nooit CR of LF sturen. Commando's uit een gegevensbestand, met een **verbodslijst** (`KS00;` zet de QMX in straight key-modus, `PS0;` zet hem uit).
- Volgorde: eerst Icom (stap 4), QMX na het Icom-prototype (stap 4b, zodra er een toestel is). Zo toetsen we de `Rig`-trait aan twee echt verschillende radio's.
- Later: Yaesu en een Hamlib-brug als extra implementaties van `Rig`.
- **Poll-reserve**: elke 500 ms frequentie, modus en TX-status, elke 250 ms S-meter, naast de ongevraagde transceive-berichten.
- **Gedrag bij wegvallen**: kabel eruit of radio uit: status "Niet verbonden" en elke 2 s opnieuw proberen. Geen crash, geen blokkerende wachtrij.

### 3.5 Zenden (keyer)

Doel: tekst veilig naar de radio, zonder dat de radio zelf een wachtrij nodig heeft.

1. **Opschonen**: hoofdletters, ongeldige tekens eruit, prosigns `<AR>` omzetten naar `^AR` [TEST].
2. **Knippen** in blokken van **maximaal 30 tekens op woordgrenzen** (nooit midden in een woord of een prosign). Een enkel woord langer dan 30 tekens wordt geweigerd met een melding.
3. **Duur schatten** met PARIS: eenheid = 1,2 / WPM seconden. Een blok van N eenheden duurt N x eenheid.
4. **Eén blok tegelijk** sturen. Het volgende blok wordt ongeveer **één woordpauze vóór het einde** van het vorige gestuurd, zodat er geen hoorbaar gat komt. Of de radio een tweede 0x17 tijdens het zenden aanvaardt, is een **[TEST]**. Valt dat tegen, dan wachten we op "TX uit" voor het volgende blok en accepteren we een korte pauze tussen blokken.
5. **Controle**: bij het sturen van een blok moet de TX-status binnen 500 ms op TX springen. Zo niet: melding "Radio zendt niet. Staat Break-in aan en de mode op CW?".
6. **Noodstop**: `FF` naar de radio, wachtrij leeg, status "Gestopt". Deze route omzeilt alle wachtrijen.
7. **Veiligheid (uit hoofdstuk 12)**: maximumduur per bericht (instelbaar, standaard 60 s), **geen automatisch zenden zonder opdracht van de operator**: de app zendt alleen na een klik of toets van Kristof. Geen auto-antwoorden, geen "contest-robot".
8. **Snelheid**: wijzigen in de app stuurt `14 0C`. Schaal [TEST]. Wijzigt de gebruiker de snelheid op de radio, dan volgt de regelaar in de app (transceive of poll).

**Paddle in het gesprek (zie 2.8):** `RigEvent` krijgt later een gebeurtenis `PaddleText(String)` voor radio's die de gesleutelde tekst kunnen doorgeven (QMX `TB`). Voor de Icom komt de tekst uit de audio-decoder.

**Strategie per radio** (gekozen via `capabilities().cw_text`):

- **Icom (`Chunked`)**: blokken van maximaal 30 tekens, zoals hierboven.
- **QMX (`Buffered`)**: de radio heeft een buffer van 80 tekens en meldt zelf met `KY;` hoe vol die is (`KY0;` tot 75% vol, `KY1;` meer dan 75%, `KY2;` leeg). CWLD geeft tekst **per woord** door, wacht bij `KY1;` en vult bij `KY0;` bij. Een bericht dat niet meer past wordt door de radio volledig genegeerd (`?;`), dus nooit een woord sturen als het niet zeker past. De noodstop moet op de QMX eerst uitgetest worden (geen stopcommando in de gewone modus gedocumenteerd).
- **Snelheid**: CWLD stuurt nooit snelheid 0 naar een QMX.

**Zendslot (TxGuard)** voor de modi LUISTEREN en OEFENEN (2.7):

- Een omhulsel `GuardedRig` rond elk stuurprogramma, met een gedeelde `TxPolicy { Live, ListenOnly, Practice }`.
- **Whitelist in plaats van blacklist**: buiten LIVE zijn alleen commando's toegestaan die als veilig zijn gemarkeerd: status en frequentie lezen, frequentie en modus zetten, snelheid zetten, en de **noodstop** (`FF` bij Icom, de stopvarianten bij QMX). Alles wat niet op de lijst staat wordt geweigerd. Daardoor is een nieuw of vergeten commando standaard geblokkeerd.
- Geblokkeerd buiten LIVE: tekst zenden (Icom 0x17, QMX `KY`), PTT en zenden (QMX `TQ1;`, `TX;`, `KD1;`), en DTR-keying.
- In OEFENEN gaat de uitvoer van de keyer niet naar de radio maar naar een `SidetoneSink` (toon via de pc-luidsprekers).
- De lijst van veilige commando's staat per radio in het commandobestand (3.4), zodat elk stuurprogramma er zelf voor moet zorgen.
- **Test**: een eenheidstest met de radiosimulator controleert dat in LUISTEREN en OEFENEN **nul** zendcommando's aankomen, ook als de interface er één probeert te sturen.

### 3.6 Audio en DSP

- **Ingang**: "USB Audio CODEC" van de radio, 48 kHz mono. Resample naar 8 kHz voor decodering, 12 kHz of 48 kHz blijft beschikbaar voor de luisterketen.
- **Waterfall**: FFT 1024 punten, venster Hann, 20 rijen per seconde, breedte begrensd tot ongeveer 100 tot 1200 Hz rond de CW-pitch.
- **Signaaldetectie** (voor de lijst): pieken in het gemiddelde spectrum boven een drempel, minstens 3 s stabiel, met een "toetsbaarheids"-controle (aan/uit patroon).
- **Decodering van het gekozen signaal**: mengen naar 0 Hz met de gevolgde toon, laagdoorlaat, envelope, dan de decoder. AFC volgt de toon binnen een instelbaar venster.
- **Luisterketen** is **aparte keten** met eigen uitgang (de koptelefoon van de pc), niet verwarren met de decoderketen. Voor stap 6.
- **Bibliotheken**: `cpal` (audio), `rustfft` (FFT), `ringbuf` of gelijkwaardig. Licenties vóór stap 8 definitief nakijken (voorlopig: cpal Apache-2.0, rustfft MIT of Apache-2.0, beide verenigbaar met GPL-3.0) **[VERIFIEER]**.

### 3.7 Decoder (interface; algoritme komt in stap 5)

```rust
pub struct DecoderConfig { pub wpm_min: f32, pub wpm_max: f32, pub language_model: bool /* ... */ }

pub trait SignalDecoder {
    /// Voer een blok envelope-samples in, met zachte waarschijnlijkheid dat de sleutel neer is.
    fn push(&mut self, key_prob: &[f32]);
    /// Haal nieuwe, bevestigde tekens en de huidige onzekere staart op.
    fn poll(&mut self) -> DecodeUpdate;
}

pub struct DecodeUpdate {
    pub committed: String,              // vaste tekst
    pub tentative: Vec<TentativeChar>,  // onzekere staart met alternatieven
    pub wpm: f32,
    pub snr_db: f32,
    pub confidence: f32,
}
pub struct TentativeChar { pub best: char, pub alts: Vec<(char, f32)> }
```

De UI hoeft de interne werking niet te kennen. Daardoor kan de decoder in stap 5 volledig worden vervangen zonder dat de interface verandert, en kan de testbank dezelfde interface gebruiken.

**Bron "radio-decoder"**: sommige radio's (QMX met `TB;`) leveren zelf gedecodeerde tekst, zonder waarschijnlijkheden en zonder tijdstippen. CWLD toont die als een tweede bron in het gesprek en gebruikt ze als referentie in de testbank. Ze vervangt de eigen decoder nooit, maar is de noodoplossing als een radio geen bruikbare CW-audio via USB geeft.

### 3.8 Log4OM-koppeling (cwld-udp), herzien in versie 2

**Correctie op versie 1:** het WSJT-X-formaat was als standaard voorgesteld. Dat is een omweg voor CW: WSJT-X is FT8-software en CWLD heeft er niets mee te maken. Alleen het **UDP-berichtenformaat** is nuttig omdat Log4OM het begrijpt. Voor het loggen zelf is er een betere weg. Daarom nu vier wegen, kiesbaar in Instellingen, sectie Log4OM:

| Weg | Wat CWLD doet | Voordeel | Nadeel |
|---|---|---|---|
| **1. ADIF MESSAGE (standaard)** | Bij "Log QSO" één ADIF-record via UDP naar de Log4OM-service "ADIF MESSAGE" (Log4OM-handleiding noemt voor ADIF poort 2333 **[VERIFIEER]**) | Geen FT8-formaat, modus CW staat gewoon in het ADIF-record, eenvoudig te testen | Log4OM krijgt tijdens het QSO geen live roepletters of frequentie in zijn invoerveld |
| **2. Live velden via JT-status (optie)** | Stuurt status-berichten (type 0 en 1) zodat Log4OM bij "JT MESSAGE" roepletters, band, frequentie en modus in zijn velden zet | Live in Log4OM | Formaat is voor FT8. Of Log4OM de modus "CW" aanvaardt is een **[TEST]**. Niet gelijktijdig met weg 1 voor loggen. |
| **3. Log4OM met eigen CAT (optie)** | Niets: Log4OM verbindt zelf met de radio. Icom: CWLD op Serial Port A, Log4OM op Serial Port B (USB (B) Function op CI-V). | Bewezen weg, geen UDP-omweg | Meer instellingen. QMX op Windows 11 heeft maar twee virtuele poorten. |
| **4. N1MM MESSAGE (later, voor contests)** | ContactInfo-berichten met uitwisselingsvelden | Past bij contestgebruik | Formaat nog uit te zoeken |

**Advies:** starten met **weg 1**. Weg 2 of 3 pas toevoegen als Kristof de live velden in Log4OM mist. Er geldt altijd: **nooit twee wegen tegelijk laten loggen**, anders komt een QSO dubbel binnen (zie bevindingen 3.1).

- **ADIF-record** (voorbeeldvelden): `CALL`, `QSO_DATE`, `TIME_ON`, `TIME_OFF`, `BAND`, `FREQ`, `MODE=CW`, `RST_SENT`, `RST_RCVD`, `NAME`, `QTH`, `GRIDSQUARE`, `STATION_CALLSIGN`, `OPERATOR`, `TX_PWR`, `COMMENT`. Het ADIF-formaat is een openbare specificatie.
- **Poort en adres**: instelbaar, standaard 127.0.0.1. Elke bron moet een eigen poort hebben (Log4OM-handleiding).
- **Optioneel later**: Log4OM Remote Control (`<WORKED>call`) om "al gewerkt?" in de badges te tonen **[VERIFIEER]** de volledige berichtenlijst.
- **[TEST]**: ADIF MESSAGE-poort en welk veld Log4OM aanvult, plus de vraag of weg 2 met modus CW werkt.

### 3.9 Instellingen en gegevensopslag

- Eén JSON-bestand `settings.json` in de gebruikersmap (Windows: `%APPDATA%\CWLD`, Linux: `~/.config/cwld`), met `schema_version` en migraties.
- `chips.json`: lijst chips met `id`, `label_en`, `label_nl`, `text`, `hotkey`, `direct_send`, `order`.
- `conversations/YYYY-MM-DD.jsonl`: gesprekken (lokaal).
- Geen account, geen cloud, geen telemetrie. De updatecontrole stuurt alleen de huidige versie en het platform, geen persoonsgegevens (zoals vastgelegd in hoofdstuk 12).

Voorbeeld `settings.json` (ingekort):

```json
{
  "schema_version": 1,
  "station": { "calls": ["ON3VZ", "ON3KC", "ON3KRB"], "name": "Kristof", "qth": "Hoboken", "locator": "JO20EE" },
  "radio":   { "driver": "icom_civ", "model": "ic7300mk2", "address": "B6", "port": "auto" },
  "audio":   { "input": "USB Audio CODEC", "sample_rate": 48000 },
  "keyer":   { "max_message_seconds": 60, "default_wpm": 22, "chip_direct_send_default": false },
  "log4om":  { "enabled": true, "host": "127.0.0.1", "port": 2333, "mode": "adif_message" },
  "dictionary": { "show": "tooltip", "explain_language": "app" },
  "ui":      { "language": "nl", "theme": "dark", "signals_panel": false }
}
```

### 3.10 UI-architectuur (Svelte 5)

```
ui/src/
  lib/
    api.ts                 Tauri-commando's en events, typed
    stores/                $state-runes: radio, conversation, signals, chips, settings
    i18n.ts                laadt en.json of nl.json, wisselt zonder herstart
  components/
    StatusBar.svelte  Waterfall.svelte  Conversation.svelte  Bubble.svelte
    QsoPanel.svelte   ChipBar.svelte    Composer.svelte      SignalsPanel.svelte
    Banner.svelte     SettingsForm.svelte  Wizard.svelte     Diagnostics.svelte
  routes/ (of eenvoudige view-switch)
```

- **Svelte 5 runes** (`$state`, `$derived`, `$effect`). Eén store per domein, geen globale "alles-in-één".
- **Talen**: sleutels zoals `status.connected`, `composer.send`. Geen vaste tekst in componenten. Test in de CI: elke sleutel bestaat in zowel `en.json` als `nl.json`.
- **Thema's**: CSS-variabelen (zoals in het voorbeeldbestand), donker, licht en "volg systeem".

**Tauri-commando's (UI naar kern)**, ruwe lijst:

`list_ports`, `connect_radio`, `disconnect_radio`, `set_frequency`, `set_cw_speed`, `send_text`, `stop_tx`, `select_signal`, `confirm_char`, `log_qso`, `get_settings`, `save_settings`, `list_chips`, `save_chips`, `check_for_update`, `install_update`, `start_diagnostics`.

**Events (kern naar UI)**: `radio:frequency`, `radio:mode`, `radio:tx`, `radio:smeter`, `radio:speed`, `radio:status`, `decode:update`, `signals:list`, `keyer:queue`, `log4om:status`, `app:warning`, plus de waterfall-`Channel`.

---

### 3.11 Eigen logboek en ADIF (nieuw in versie 3)

Doel: CWLD kan **zelfstandig loggen**, ook zonder Log4OM, en maakt zelf **correcte ADIF**, ook voor fauna en flora (WWFF, in België ONFF). Log4OM blijft optioneel (3.8).

**A. Opslag**
- Eigen bestand `logbook.db` (SQLite) in de gebruikersmap, met tabellen `qso`, `station_profile` en `activation`. Reden: betrouwbaar bij stroomuitval of een crash, snel zoeken en dubbele-QSO-controle. ADIF is het **uitwisselformaat**, niet de masterbron. Licentie van de gekozen SQLite-bibliotheek vóór stap 8 nakijken **[VERIFIEER]**.
- QSO's worden nooit stilzwijgend gewist. Verwijderen vraagt bevestiging, bewerken is toegestaan.

**B. Stationsprofielen** (kiesbaar in de statusbalk, nooit stilzwijgend wisselen)

| Profiel | Voorbeeld |
|---|---|
| **Thuis** | `STATION_CALLSIGN` ON3VZ, `OPERATOR` ON3VZ, grid JO20EE, IC-7300MK2, standaardopmerking leeg |
| **Portable** | `STATION_CALLSIGN` ON3VZ/P, `OPERATOR` ON3VZ, grid **per activatie**, `TX_PWR` 5, QRP Labs QMX, standaardopmerking "5W QRP" |

Een profiel bevat: roepletters op de lucht, operatorroepletters, standaard-grid, vermogen, radio, antenne en een standaardopmerking. De standaardopmerking komt overal waar het kan in het ADIF-veld `COMMENT`.

**C. Activaties (fauna en flora)**
- Knop "Nieuwe activatie": programma (**WWFF**, POTA, SOTA of geen), referentie, grid van deze locatie, vermogen en notitie.
- **Controle op de referentie**: het ADIF-formaat voor een WWFF-referentie is `xxFF-nnnn`, met `nnnn` altijd vier cijfers met voorloopnullen (bv. `ONFF-0599`). CWLD weigert een andere vorm.
- Een activatie kan **meer dan één programma** tegelijk hebben (bv. WWFF en POTA). Elk programma krijgt zijn eigen velden (zie D).
- **Teller** van het aantal QSO's in de activatie, met de WWFF-drempel (volgens mijn kennis 44 QSO's, de officiële pagina die ik las vermeldt dat niet, dus **[VERIFIEER]** bij de ONFF-coördinator) en een waarschuwing als de activatie over middernacht UTC loopt.
- **Dubbele QSO's**: zelfde roepletters, band en modus binnen de activatie worden gemarkeerd, maar niet geblokkeerd.

**D. ADIF-schrijver: wat erin komt**

Bron voor het WWFF-gedeelte: de pagina "Confirming and sending LOG" van WWFF en de ADIF-specificatie (WWFF-velden bestaan sinds ADIF 3.1.3). Verplicht volgens WWFF: `CALL`, `QSO_DATE`, `TIME_ON`, `BAND`, `MODE`. WWFF zegt dat alle andere velden genegeerd worden, maar voor de activator vraagt WWFF ook `STATION_CALLSIGN`, `OPERATOR`, `MY_SIG` en `MY_SIG_INFO`.

| Veld | Voorbeeld | Opmerking |
|---|---|---|
| `CALL` | DK1AB | hoofdletters, `/P` blijft staan |
| `QSO_DATE`, `TIME_ON` | 20261103, 140300 | altijd **UTC**, tijd met 6 cijfers |
| `QSO_DATE_OFF`, `TIME_OFF` | | einde van het QSO |
| `BAND` | 40M | |
| `FREQ` | 7.025400 | in MHz |
| `MODE` | CW | geen `SUBMODE` bij CW |
| `RST_SENT`, `RST_RCVD` | 599 | |
| `NAME`, `QTH`, `GRIDSQUARE` | Hans, Hamburg, JO43 | van het tegenstation |
| `STATION_CALLSIGN` | ON3VZ/P | roepletters op de lucht |
| `OPERATOR` | ON3VZ | persoonlijke roepletters |
| `MY_GRIDSQUARE` | JO20QE | **per activatie**, 6 tekens |
| `TX_PWR` | 5 | in watt |
| `MY_RIG`, `MY_ANTENNA` | QRP Labs QMX | |
| `COMMENT` | 5W QRP | uit het profiel |
| `MY_SIG` | WWFF | wat WWFF vraagt |
| `MY_SIG_INFO` | ONFF-0599 | wat WWFF vraagt |
| `MY_WWFF_REF` | ONFF-0599 | ADIF-standaardveld, we schrijven **beide** |
| `SIG`, `SIG_INFO`, `WWFF_REF` | WWFF, ONFF-0123 | **enkel bij park-to-park**: referentie van het tegenstation |
| `MY_POTA_REF`, `POTA_REF` | | alleen als POTA in de activatie zit (ADIF 3.1.4). Voor POTA-P2P het eigen veld gebruiken, **niet** `SIG`. |
| `MY_SOTA_REF`, `SOTA_REF` | | idem voor SOTA |
| `APP_CWLD_...` | | eigen velden, door andere programma's genegeerd |

Regels voor het bestand:
- **ADI-formaat**: elk veld als `<NAAM:lengte>waarde`, record afgesloten met `<EOR>`, bestandskop afgesloten met `<EOH>`.
- **Alleen ASCII.** De "INTL"-velden voor accenten bestaan volgens de specificatie enkel in ADX-bestanden, dus in ADI worden accenten weggewerkt (`Hobokèn` wordt `Hoboken`). ADX-export kan later.
- **ADIF-versie in de kop**: de laagste versie die alle gebruikte velden dekt (`3.1.4` zodra POTA-velden voorkomen). De huidige ADIF-versie is 3.1.7 (maart 2026) **[VERIFIEER]** welke versie Log4OM en de ONFF-coördinator aankunnen.
- **Bestandsnaam voor WWFF**: `{roepletters} @ {referentie} {datum}.adi`, waarbij `/` in de roepletters een `-` wordt en de datum `JJJJMMDD` is. Voorbeeld: `ON3VZ-P @ ONFF-0599 20261103.adi`. Bij een activatie over meerdere dagen geldt de eerste dag.
- Een WWFF-controle zou volgens een forumdiscussie `SIG`-velden die niet "WWFF" bevatten negeren. Dat is door WWFF niet bevestigd, dus houden we ons aan de regel hierboven.

Voorbeeld van een bestand (lengtes door CWLD berekend, hier door mij met een script gecontroleerd):

```
<ADIF_VER:5>3.1.4 <PROGRAMID:4>CWLD <PROGRAMVERSION:5>0.1.0 <EOH>
<CALL:5>DK1AB <QSO_DATE:8>20261103 <TIME_ON:6>140300 <BAND:3>40M <FREQ:8>7.025400 <MODE:2>CW <RST_SENT:3>599 <RST_RCVD:3>599 <NAME:4>Hans <QTH:7>Hamburg <STATION_CALLSIGN:7>ON3VZ/P <OPERATOR:5>ON3VZ <MY_GRIDSQUARE:6>JO20QE <TX_PWR:1>5 <MY_RIG:12>QRP Labs QMX <COMMENT:6>5W QRP <MY_SIG:4>WWFF <MY_SIG_INFO:9>ONFF-0599 <MY_WWFF_REF:9>ONFF-0599 <EOR>
<CALL:5>DK1AB <QSO_DATE:8>20261103 <TIME_ON:6>140300 <BAND:3>40M <FREQ:8>7.025400 <MODE:2>CW <RST_SENT:3>599 <RST_RCVD:3>599 <NAME:4>Hans <QTH:7>Hamburg <STATION_CALLSIGN:7>ON3VZ/P <OPERATOR:5>ON3VZ <MY_GRIDSQUARE:6>JO20QE <TX_PWR:1>5 <MY_RIG:12>QRP Labs QMX <COMMENT:6>5W QRP <MY_SIG:4>WWFF <MY_SIG_INFO:9>ONFF-0599 <MY_WWFF_REF:9>ONFF-0599 <SIG:4>WWFF <SIG_INFO:9>ONFF-0123 <WWFF_REF:9>ONFF-0123 <EOR>
```

**E. Controle vóór export**

Een venster toont wat er geëxporteerd wordt en blokkeert op fouten:
1. `STATION_CALLSIGN` komt overeen met het gekozen profiel.
2. `MY_GRIDSQUARE` aanwezig, 6 tekens, en **getoond in grote letters** (je ziet dus welke locatie je export vermeldt). Reden: een log dat met het grid van een vorige locatie wordt ondertekend geeft een verkeerde LoTW-locatie.
3. Referentie voldoet aan `xxFF-nnnn`.
4. Elk QSO heeft `CALL`, datum, tijd, band, modus en RST.
5. Datum en tijd zijn UTC en liggen binnen de activatie.
6. Dubbele QSO's staan in een lijstje ter controle.

**F. Delen met andere programma's**
- **Exporteer activatie**, **Exporteer alles**, **Exporteer sinds laatste export**. Dat laatste past bij de bestaande werkwijze met PoLo: export en handmatig importeren in Log4OM.
- Optie "Live aan een .adi-bestand toevoegen" na elk QSO, zodat een ander programma het bestand kan inlezen.
- Optie "Direct naar Log4OM via UDP" (3.8, weg 1).
- Elk QSO onthoudt waarheen het al verstuurd is (`exported_to`: bestand, Log4OM). Zo wordt een QSO **niet dubbel** verstuurd. Log4OM controleert wel op dubbele QSO's, maar de log vult zich met meldingen.
- **Importeren** van een ADIF-bestand (bv. een export uit Log4OM) voor de dubbele-QSO-controle. Pas na de MVP.
- **Niet in de MVP**: rechtstreeks uploaden naar QRZ, LoTW, Club Log of de WWFF-coördinator. Dat blijft het werk van Log4OM.

## 4. Testen

| Laag | Aanpak |
|---|---|
| **cwld-radio** | Eenheidstests met opgenomen CI-V-bytereeksen. **Radiosimulator** (een nepsysteem met een virtuele poort of in-memory kanaal) die frames beantwoordt, echo's en transceive nabootst. Daarmee kan het hele programma zonder radio worden getest. |
| **cwld-keyer** | Eenheidstests voor opschonen, knippen op woordgrenzen, prosigns, duurschatting en de noodstop. |
| **cwld-udp** | Tests van het ADIF-record (veldformaat, lengtes) en, als weg 2 gekozen wordt, byte-voor-byte tests van de JT-berichten. Bij voorkeur ook een echte test met Log4OM (**[TEST]**). |
| **cwld-keyer, GuardedRig** | In LUISTEREN en OEFENEN: nul zendcommando's bij de radiosimulator, ook bij pogingen via de interface. In OEFENEN: zijtoon met de juiste timing. De kopieeroefening vergelijkt de ingetypte tekst met de gekende tekst. |
| **cwld-log** | ADIF-schrijver: lengtes van de velden, UTC-tijden, ASCII-regel, referentievorm `xxFF-nnnn`, bestandsnaam. Terug inlezen met onze eigen lezer en **vergelijken met een echte PoLo- en Log4OM-export** van dezelfde QSO's. Import in Log4OM door Kristof. |
| **cwld-dsp, cwld-decoder** | Testbank (stap 3): tekenfoutpercentage per SNR en snelheid. Regressiegrens in de CI. |
| **UI** | Component-tests (Vitest), een paar end-to-end tests met Playwright tegen de radiosimulator, i18n-sleutelcontrole. |
| **Hardware** | Handmatige testlijst op Kristofs radio, de [TEST]-punten uit de bevindingen, **eerst op laag vermogen of in een dummy load**. |

---

## 5. Risico's en wat ermee gebeurt

| Risico | Gevolg | Aanpak |
|---|---|---|
| 0x17 gedraagt zich anders dan verwacht (wachtrij, prosigns) | Zendtekst haperend | Metingen in stap 4 voor het ontwerp van de keyer vastligt. Reserve: DTR-keying via USB. |
| Paddle niet zichtbaar in de app | Geen eigen tekst in het gesprek bij paddle-gebruik | TX-indicator als minimum. Audio-test. |
| Verschoven MK2-commando's | Verkeerde instelling uitgelezen of gezet | Commandotabel uit de MK2-guide in een gegevensbestand. Schrijfopdrachten naar menu-instellingen komen **niet** in de MVP: de app leest en adviseert, de gebruiker zet de instelling zelf op de radio. |
| Decoderkwaliteit | Kernbelofte van het project | Testbank eerst. Interface stabiel, algoritme vervangbaar. |
| Windows SmartScreen | Vertrouwen van nieuwe gebruikers | Duidelijke downloadpagina, later SignPath. |
| Log4OM-berichten niet zoals verwacht | QSO's komen dubbel of niet binnen | Eén berichttype tegelijk, [TEST] met echte Log4OM, instelbaar. |
| QMX levert geen CW-audio via USB | Eigen decoder werkt niet op de QMX | Eerste test met een echte QMX. Reserve: radio-decoder via `TB;`. Zie `bevindingen_qmx.md`. |
| Stoppen van een QMX-zending werkt niet via CAT | Geen echte noodstop | Uittesten, anders duidelijke melding en paddle als afbreker |
| Svelte-ecosysteem kleiner dan React | Minder kant-en-klare componenten | Interface is overzichtelijk, we bouwen de weinige componenten zelf. |

---

## 6. Wat dit betekent voor stap 3 en 4

**Stap 3 (testbank, Opus)** kan starten zonder UI, in `cwld-testbench` en `cwld-decoder` met de interface uit 3.7.

**Stap 4 (prototype, Sonnet)**, uitgewerkt tot een korte lijst:
1. Workspace opzetten, `cwld-radio` met de radiosimulator.
2. Poort zoeken en verbinden, live frequentie, modus, TX en S-meter in de statusbalk.
3. Keyer met tekstvak, 30-tekenblokken, snelheid, **noodstop**.
4. Eén chip (bv. "CQ") met variabelen.
5. UDP-status en QSO Logged naar Log4OM.
6. Diagnosescherm met de [TEST]-metingen uit bevindingen paragraaf 8.

Zonder decoder en zonder waterfall. Die komen in stap 5 en 6.

**Stap 4b (zodra er een QMX is):** tweede stuurprogramma `kenwood_ts480`, de QMX-tests uit `bevindingen_qmx.md`. **Woordenboek**: gegevensbestand `dictionary.json`, aanvullen in het tekstvak en het afkortingenpaneel horen bij stap 7 (integratie en polish), de basis van het woordenboek kan in stap 4 al in de chips zitten.

---

## 7. Open punten voor Kristof

1. **Groot scherm (B)** nu of later? Voorstel: later.
2. **Signalenpaneel (C)**: eerst alleen detectie en keuze, meerdere decoders tegelijk later? Voorstel: ja.
3. **Chips standaard "in het tekstvak"** of "direct zenden"? Voorstel: in het tekstvak.
4. **Roepletterlijst** voor controle: bron en licentie uitzoeken in stap 5.
5. ~~QMX voor de tests?~~ **Beantwoord:** ja, een recente QMX, Windows 11 (dus twee virtuele COM-poorten). Nog te noteren: firmwareversie.
6. ~~Extra Belgische afkortingen?~~ **Beantwoord:** nee, standaardwoordenboek volstaat.
7. **Uitleg-taal** bij het lezen van Q-codes: standaard gelijk aan de app-taal. Akkoord?
8. ~~Startmodus~~ **Beantwoord:** CWLD start in de laatst gebruikte modus. Was dat LIVE, dan start het in LUISTEREN en vraagt LIVE een bewuste klik (veiligheidsregel uit 2.7).
9. **Referentiebestanden voor het logboek**: kun je een echte **PoLo-export** en een **Log4OM-export** van een paar WWFF-QSO's (ADI-bestanden) in het project zetten? Dan vergelijk ik onze uitvoer er veld voor veld mee.
10. **Thuis versus portable**: klopt het dat thuis `ON3VZ` en grid JO20EE geldt en portable `ON3VZ/P` met een grid per activatie en 5 W? En wil je de opmerking "5W QRP" ook thuis bij QRP-QSO's?

---

## Bijlage A: startinhoud woordenboek

Dit is de **ontwerpversie** van de lijst. De tweetalige gegevensfile wordt gemaakt in de bouwfase. De betekenissen volgen de gangbare ITU- en ARRL-omschrijvingen **[VERIFIEER]** voor de uiteindelijke versie, vooral de Nederlandse vertalingen.

### A.1 Q-codes

Elk item heeft een vraagvorm (met `?`) en een mededelingsvorm (zonder `?`).

| Code | Vraag (met ?) | Mededeling (zonder ?) |
|---|---|---|
| QRA | Wat is de naam van uw station? | De naam van mijn station is ... |
| QRB | Hoe ver bent u van mijn station? | Onze stations zijn ongeveer ... km van elkaar |
| QRG | Kunt u mijn exacte frequentie opgeven? | Uw exacte frequentie is ... |
| QRH | Varieert mijn frequentie? | Uw frequentie varieert |
| QRI | Hoe is de toon van mijn zending? | De toon van uw zending is ... (1 goed, 2 wisselend, 3 slecht) |
| QRK | Hoe goed is mijn leesbaarheid? | Uw leesbaarheid is ... (1 tot 5) |
| QRL | Bent u bezet? | Ik ben bezet (de frequentie is in gebruik) |
| QRM | Hebt u storing van andere stations? | Ik heb storing van andere stations |
| QRN | Hebt u last van atmosferische storing? | Ik heb last van atmosferische storing |
| QRO | Moet ik meer vermogen gebruiken? | Gebruik meer vermogen |
| QRP | Moet ik minder vermogen gebruiken? | Gebruik minder vermogen (ook: zwakstroom, laag vermogen) |
| QRQ | Moet ik sneller seinen? | Sein sneller |
| QRS | Moet ik langzamer seinen? | Sein langzamer |
| QRT | Moet ik stoppen met zenden? | Ik stop met zenden (sluit het station) |
| QRU | Hebt u iets voor mij? | Ik heb niets voor u |
| QRV | Bent u gereed? | Ik ben gereed |
| QRX | Moet ik wachten? | Wacht (ik roep u later weer op ...) |
| QRZ | Wie roept mij? | U wordt geroepen door ... |
| QSA | Hoe sterk is mijn signaal? | Uw signaalsterkte is ... (1 tot 5) |
| QSB | Fadt mijn signaal? | Uw signaal fadt |
| QSD | Is mijn seinen slecht? | Uw seinen is slecht |
| QSK | Hoort u mij tussen uw tekens (break-in)? | Ik hoor u tussen mijn tekens, u mag onderbreken |
| QSL | Kunt u ontvangst bevestigen? | Ik bevestig de ontvangst (ook: QSL-kaart) |
| QSM | Moet ik het laatste bericht herhalen? | Herhaal het laatste bericht |
| QSO | Kunt u rechtstreeks met ... werken? | Ik kan rechtstreeks met ... werken (ook: een contact) |
| QSP | Wilt u doorgeven aan ...? | Ik geef door aan ... |
| QSX | Wilt u ... beluisteren op ...? | Ik luister naar ... op ... |
| QSY | Moet ik van frequentie veranderen? | Verander van frequentie |
| QTC | Hoeveel berichten hebt u? | Ik heb ... berichten voor u |
| QTH | Wat is uw locatie? | Mijn locatie is ... |
| QTR | Hoe laat is het precies? | Het is precies ... |

### A.2 CW-afkortingen

| Afk. | Engels | Nederlands |
|---|---|---|
| ABT | about | over, ongeveer |
| AGN | again | opnieuw |
| ANT | antenna | antenne |
| BK | break | onderbreking |
| BTW | by the way | trouwens |
| CFM | confirm | bevestig |
| CL | closing station | ik sluit af |
| CPY | copy | gelezen, ontvangen |
| CQ | general call | algemene oproep |
| CUAGN | see you again | tot weerziens |
| CUL | see you later | tot later |
| DE | from, this is | van |
| DR | dear | beste |
| DX | long distance | ver verkeer |
| ES | and | en |
| FB | fine business | uitstekend |
| GA | go ahead | ga door |
| GE | good evening | goedenavond |
| GL | good luck | succes |
| GM | good morning | goedemorgen |
| GN | good night | goedenacht |
| HI | laughter | lach |
| HPE | hope | hoop |
| HR | here | hier |
| HW | how | hoe (`HW?` hoe ontvangt u mij?) |
| MNI | many | veel |
| NR | number | nummer |
| OM | old man | collega-amateur (man) |
| OP | operator | operator |
| PSE | please | alsjeblieft |
| PWR | power | vermogen |
| R | received | ontvangen, begrepen |
| RIG | transceiver | radio |
| RPT | repeat | herhaal |
| RX | receiver | ontvanger |
| SIG | signal | signaal |
| SRI | sorry | sorry |
| TKS, TNX | thanks | dank |
| TU | thank you | dank u |
| TX | transmitter | zender |
| UFB | ultra fine business | fantastisch |
| UR | your | uw, jouw |
| VY | very | heel |
| WID | with | met |
| WKD | worked | gewerkt |
| WX | weather | weer |
| XYL | wife | echtgenote |
| YL | young lady | dame (amateur) |
| 73 | best regards | vriendelijke groeten |
| 88 | love and kisses | liefs |

### A.3 Prosigns

| Prosign | Morse | Engels | Nederlands |
|---|---|---|---|
| AR | `.-.-.` | end of message | einde bericht |
| AS | `.-...` | wait | wacht |
| BK | `-...-.-` | break | onderbreking |
| BT | `-...-` | pause, new paragraph | pauze, nieuwe alinea |
| CT of KA | `-.-.-` | start of transmission | begin zending |
| HH | `........` | error | vergissing |
| K | `-.-` | go ahead | ga door (iedereen) |
| KN | `-.--.` | go ahead, named station only | ga door, alleen het genoemde station |
| SK | `...-.-` | end of contact | einde contact |
| SN of VE | `...-.` | understood | begrepen |

### A.4 Rapporten en contest-afkortingen

- **RST**: R leesbaarheid (1 tot 5), S signaalsterkte (1 tot 9), T toon (1 tot 9). `599` is het uitstekend rapport.
- **Cut numbers** (contest): `N` = 9, `T` = 0, `A` = 1. Dus `5NN` = 599.
- **Snelkeuze**: 599, 579, 559, 449.
