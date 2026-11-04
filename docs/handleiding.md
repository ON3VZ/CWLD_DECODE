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
