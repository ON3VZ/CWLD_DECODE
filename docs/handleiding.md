# CWLD, korte handleiding (testversie)

## 1. Eerste start in vier stappen

1. **Radio kiezen.** Klik rechtsboven op het tandwiel. Kies bij Radiotype je radio (bijvoorbeeld Icom IC-7300 MK II) en bij Poort de **Serial Port A (CI-V)**. Windows toont twee poorten voor de IC-7300 MK II, de tweede is voor het zenden met de sleutel. Klik op **Connect**. Draai aan de draaiknop van de radio: de frequentie in CWLD moet meelopen. Zet daarvoor op de radio onder MENU, SET, Connectors, CI-V de optie **CI-V Transceive** op ON.
2. **Audio kiezen.** Scrol in hetzelfde venster naar **Audio en decoder**. Kies bij Audio-ingang de **USB Audio CODEC** van de radio (in Windows heet die soms "Microfoon (USB Audio Device)"). CWLD start de audio meteen opnieuw met die ingang. Kijk onder "Luistert" of daar nu de ingang van de radio staat. Staat daar nog de microfoon van je laptop, klik dan op **Opnieuw starten met deze instellingen**.
3. **Roepletters invullen** bij Station.
4. **Test.** Stem af op een CW-signaal. De niveaubalk moet bewegen en in de waterfall zie je het signaal.

## 2. Windows en de microfoon

Windows laat een app alleen naar USB-audio luisteren als de microfoontoegang aan staat. Klik op Start, typ `microfoon` en kies Privacy-instellingen voor microfoon. Zet **Toegang tot de microfoon**, **Apps toegang geven tot uw microfoon** en **Desktop-apps toegang geven tot uw microfoon** aan. Zet in Windows ook alle geluidsverbeteringen voor deze ingang uit.

## 3. De waterfall

- De decoder zoekt zelf het sterkste CW-signaal als het een tijdje stil is op zijn toonhoogte (1,5 seconde). De rode lijn springt dan mee. Dit kan uit in de instellingen: "Zelf het sterkste signaal zoeken".
- Klik op een signaal: de decoder luistert vanaf dan naar die toonhoogte (rode lijn) en blijft daar minstens 10 seconden. Zwakke signalen vindt hij niet zelf, die moet je aanklikken.
- Rechtsboven kies je zoom **1x, 2x of 4x**. Onderaan staan de frequenties in Hz.
- Zet de CW-pitch van de radio op dezelfde waarde (MENU, SET, Function, CW Pitch), dan staat een afgestemd signaal precies op de lijn.

## 4. Gesprekken en Nieuw QSO

- Elk gesprek heeft een eigen tabblad, met eigen roepletters, naam, RST en QTH. Het tabblad met het groene bolletje is het lopende gesprek. Nieuwe tekst komt altijd daar, ook als je naar een ouder tabblad kijkt.
- Een nieuw tabblad begint vanzelf als je meer dan 500 Hz van frequentie wisselt of als er een andere roepletter wordt gehoord. Met de knop **+ Nieuw QSO** begin je zelf een nieuw gesprek en wordt het scherm leeg. Het vorige gesprek blijft bewaard.
- Klik op een ouder tabblad om het terug te lezen. Zenden kan alleen in het lopende gesprek. Het kruisje op een oud tabblad verwijdert dat gesprek.
- De gesprekken blijven bewaard op je computer (hoogstens 40).

## 5. Signaal en drempel

- Naast de zekerheidsmeter staat **Signaal** (groen) of **Geen signaal** (grijs). Bij grijs wordt er niets gedecodeerd.
- Zie je tekst terwijl er geen signaal is, zet dan in de instellingen de **Drempel voor signalen** hoger (standaard 3,5, probeer 4 of 4,5). Zie je een zwak signaal niet meer, zet hem lager. Het werkt meteen.

## 6. Opnames

Klik bovenaan op **OPNAME**, laat het signaal lopen en klik op Stop. Het WAV-bestand staat in Documenten\CWLD, met de frequentie in de naam. Daarmee kan de decoder worden afgesteld, en CwGet kan het ook lezen.

## 7. Zenden veilig houden

Alleen in de stand **LIVE** zendt CWLD. Gebruik bij het testen laag vermogen en een kunstantenne of een bekend goede antenne. **Esc** stopt direct.

## 8. Updates

CWLD kijkt bij het opstarten of er een nieuwe versie is. Bovenaan verschijnt dan een balk met de knop **Bijwerken**: de app downloadt de nieuwe versie, installeert ze en start zelf opnieuw op. Je kunt ook zelf kijken: Instellingen, **Zoek updates**. Daar zie je ook wat er nieuw is. De controle haalt alleen een bestand op bij GitHub en verstuurt geen persoonsgegevens. Stop eerst met zenden voor je bijwerkt, want de app herstart.

## 9. Bandscoop

Bovenaan staat de bandscoop van de radio: hetzelfde spectrum als op het scherm van de IC-7300 MK II, met een lijn (sterkte) en een waterfall (verloop in de tijd). De rode lijn is je VFO. Beweeg met de muis over de scoop om de frequentie te zien, klik op een signaal en de radio stemt erheen.

- Hoe breed de scoop is, kies je op de radio zelf (de SPAN-knop op het scoopscherm, bv. ±25 kHz). CWLD volgt dat.
- CWLD zet de scoopgegevens zelf aan als je de radio verbindt. Op de radio moet niets extra ingesteld worden, behalve wat in de eerste start staat (CI-V via USB).
- Rechts, onder het QSO-blok, staat het blok **Audio**: wat de decoder hoort binnen het filter van de radio. Een klik daar zet de decoder op die toon.
- Beide kun je verbergen: klik onder de bandscoop op **Bandscoop verbergen**, of in het blok Audio op **Verbergen**. Je kunt ook naar Instellingen, **Beeld** gaan. Een verborgen deel haal je terug met de link bovenaan het scherm.
- De QMX heeft geen bandscoop; daar zie je alleen de audio-waterfall.

## 10. Codes en afkortingen

Naast de knop Verzenden staat een knop **Q** (of druk op F1). Er schuift rechts een paneel open met Q-codes, veelgebruikte CW-afkortingen en prosigns, elk met uitleg in het Nederlands of Engels (volgens je taalinstelling).

1. Typ in het zoekvak een code of een woord uit de uitleg. "QTH" of "locatie" vindt beide QTH.
2. Kies bovenaan Alles, Q-codes, Afkortingen of Prosigns om de lijst te beperken.
3. Klik op een code om ze aan je bericht toe te voegen. Klik op de link bij "Als vraag" om ze met een vraagteken toe te voegen (QTH? betekent: wat is uw locatie?).
4. Sluit het paneel met Esc, met het kruisje of door naast het paneel te klikken.

Prosigns komen als `<AR>`, `<SK>`, `<BT>` in je bericht. De keyer vertaalt ze voor de radio. `<CL>` staat erbij als uitleg, maar kan niet via de keyer verzonden worden.

## 11. Verbinden bij het opstarten, en als een update vastloopt

CWLD verbindt bij het opstarten zelf met de radio die je eerder koos (Instellingen, Radio, "Verbinden bij het opstarten"). Staat de radio dan nog uit, start CWLD gewoon en druk je zelf op Verbinden. Wil je dat niet, haal het vinkje weg.

Meldt de installer van een update toch "CWLD is running" of "Failed to kill CWLD" (kan nog eenmalig gebeuren bij de overstap naar 0.4.4, omdat de oude versie de nieuwe opruimstap nog niet kent):
1. Sluit CWLD met het kruisje.
2. Klik rechts op de Start-knop en kies Taakbeheer. Klik links op **Details**.
3. Klik op de kolomkop **Naam** om te sorteren. Beëindig eerst `cwld.exe` als die er nog staat: aanklikken, rechtsonder **Taak beëindigen**.
4. Zoek de `msedgewebview2.exe` met de omschrijving "WebView2: CWLD" of "WebView2 Manager" die bij CWLD hoort. Klik er met de rechtermuisknop op en kies **Procesboom beëindigen**. Alleen het ene proces beëindigen helpt niet, want WebView2 start het dan opnieuw.
5. Klik in de installer op **Cancel** en start hem opnieuw.
Lukt het nog niet: herstart de pc, open CWLD niet, en start meteen de installer.

Vanaf 0.4.4 doet de installer stap 3 en 4 zelf.
