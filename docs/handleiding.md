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

Naast de knop Verzenden staat een knop **Q** (of druk op Ctrl+Q). Er schuift rechts een paneel open met Q-codes, veelgebruikte CW-afkortingen en prosigns, elk met uitleg in het Nederlands of Engels (volgens je taalinstelling).

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


## 12. Instellingen in rubrieken

Klik rechtsboven op het tandwiel. Links staat een menu: Station, Radio en SWR, Audio, Decoder, Zenden, Weergave en taal, Contest, Updates en info. Klik op een rubriek om die te zien. Onderaan staat OK om te bewaren en Sluiten om zonder bewaren te sluiten. Schuiven en schakelaars van de decoder werken meteen, de rest geldt na OK.

## 13. Pile-up en snelle antwoorden

Onder de gewone chips staat een rij Pile-up met vier knoppen en de toetsen F1 tot F4:

- F1: je eigen roepletters.
- F2: je eigen roepletters twee keer.
- F3: R, je rapport en TU (bijvoorbeeld `R 5NN TU`). Het rapport komt uit het QSO-paneel en is standaard 5NN. Wil je 599, kies dat bij Instellingen, Zenden, Standaardrapport.
- F4: AGN?

De toetsen werken alleen als zenden mag: in LIVE of OEFENEN, niet bij LUISTEREN, en niet bij een openstaande SWR-melding. Het Q-paneel opent nu met Ctrl+Q.

## 14. Gesprekken (tabjes) en contest

- Klik op een ouder tabje: dat gesprek wordt weer het actieve, en de radio gaat naar de frequentie van dat gesprek. Tijdens het zenden of zonder radio wordt het alleen getoond.
- Gesprekken zonder nieuwe tekst sluiten na 15 minuten (instelbaar bij Weergave en taal). Het gesprek waarin je zit en gesprekken met gezonden tekst die nog niet gelogd zijn sluiten niet vanzelf. De knop Alles sluiten laat één nieuw leeg gesprek open en vraagt eerst bevestiging als er nog niet-gelogde gesprekken zijn. (Het logboek zelf komt in een volgende versie, tot dan blijft een gesprek met gezonden tekst staan tot je het zelf sluit met het kruisje.)
- Contest: klik onder de chips op Contest. Er verschijnt een chip Uitwisseling (`{CALL} {RST} {NR}`), en `{NR}` in elke chip wordt het volgnummer van drie cijfers. Zet je contest mode later weer aan terwijl er een nummer onthouden is, dan vraagt CWLD of je daar verdergaat of opnieuw begint bij 1. Het nummer telt voorlopig nog niet vanzelf op.

## 15. Decoder bijstellen

Onder het audio-waterfall (rechts) staan:

- De piekmeter: tussen 5 en 90 procent is goed. Boven 90 procent wordt de ingang van de geluidskaart overbelast, zet dan het audioniveau van de radio of de versterking lager.
- De versterking (-20 tot +20 dB) voor decoder en meter. Een opname blijft zoals ze binnenkwam.
- Zwak, Midden, Sterk: hoe streng de decoder is. Zwak probeert ook vage signalen, Sterk leest alleen duidelijke. Dit zijn standen van de drempelschuif bij Instellingen, Decoder, naar gevoel gekozen en nog niet gemeten.
- Smal of Breed: Breed heeft dubbele bandbreedte, voor driftende of tjirpende signalen. De twee rode lijnen in het waterfall tonen het filter.
- Sterkste (F6): eenmalig naar het sterkste getoetste signaal van de laatste seconden. AFC (F7): aan of uit.
- Bij Instellingen, Decoder: traagste en snelste snelheid (5 tot 60 WPM) en een vaste snelheid voor bakens.

## 16. Wedstrijdprofielen

Een profiel zet in één keer alles klaar voor een wedstrijd: de oproepknoppen (bijvoorbeeld TEST UBA) en de uitwisseling die jij zendt.

1. Klik onder de chips op **Contest** om contest mode aan te zetten. Er verschijnt een keuzelijst met het profiel.
2. Kies een profiel, bijvoorbeeld UBA DX Contest CW. Er verschijnt een korte opmerking over de regels. Klik op het kruisje om hem te sluiten.
3. Heeft het profiel meer dan één uitwisseling (bijvoorbeeld Vanuit België en Vanuit het buitenland), dan verschijnt er een tweede keuzelijst om er één te kiezen.
4. Klik op de chip **Uitwisseling** om bijvoorbeeld `PA3ABC 5NN 001 WLD` te zenden. Het volgnummer staat ook op de knop Contest.

Meegeleverd zijn: UBA DX Contest, UBA ON Contest, UBA Spring Contest, UBA Low Band Winter Contest, CQ WW DX, CQ WPX, ARRL International DX, IARU HF Championship, WAE DX en een leeg sjabloon. De regels zijn gecontroleerd op 7 november 2026. Controleer voor een wedstrijd altijd de actuele regels, vooral bij Spring en Winter, waar de bron ouder was.

**Eigen variabelen.** Open Instellingen, rubriek Contest, en kijk onder Eigen variabelen. Daar staan `SEC` (WLD), `ZONE` (14), `ITU` (27) en `PWR` (20). Je kunt ze wijzigen en er eigen bijmaken, bijvoorbeeld `AGE`. Gebruik ze als `{SEC}` in elke knop en in het zendvak. Namen als CALL, MYCALL, NAME, QTH, RST en NR zijn van CWLD zelf en kunnen niet gebruikt worden.

**Een eigen profiel maken.** Open Instellingen, rubriek Contest. Klik in de lijst op een profiel en daarna op **Kopiëren**, of klik op **Nieuw profiel**. Pas naam, oproepen, uitwisseling en het extra veld aan en klik op **Profiel opslaan**. Onder elke uitwisseling zie je hoe de tekst er straks uitziet. Een meegeleverd profiel kun je niet wijzigen.

**Verkorte cijfers.** Vink die aan in een eigen profiel als de wedstrijd het toestaat. Het volgnummer 019 wordt dan T1N.

**Het extra veld dat je ontvangt** (bijvoorbeeld de sectie van een Belgisch station) stel je hier al in. Het wordt pas gebruikt als het logboek er is.

**Reservekopie.** Onderaan de rubriek staan Dit profiel exporteren, Al mijn profielen en variabelen exporteren en Importeren uit een bestand. Een export bevat je eigen profielen en variabelen, niet de meegeleverde. Bij importeren kies je of een profiel met dezelfde naam overschreven wordt of dat er een kopie bijkomt.

**Let op bij CQ WW:** de regels rekenen het gebruik van een CW-decoder als hulpmiddel. Gebruik je CWLD, schrijf je dan in een klasse met hulp in.

## 17. Contest mode en logboek (0.7.0)

**Aanzetten.** Klik onder de chips op Contest, kies een profiel (zie hoofdstuk 16). Er verschijnt een invoerregel en daaronder acht toetsen F1 tot F8. De kleine grijze letter op elke toets is de sneltoets. Je kunt klikken of typen.

**Een QSO in Run.** Run betekent dat jij CQ roept. Druk Enter (of F1): CQ. Een station antwoordt: typ de roepletters in Call en druk Enter (of F2): je uitwisseling gaat de lucht in. Typ wat het station terugzendt: rapport (staat al op 5NN), nummer en bij Belgische stations de sectie. Space springt naar het volgende veld. Druk Enter (of F3): TU, en het QSO wordt gelogd. Ontbreekt iets, dan zendt Enter AGN.

**Een QSO in S&P.** S&P (search and pounce) betekent dat jij andere stations aanroept. Zet Ctrl+R op S&P. Typ de roepletters en druk Enter: jouw roepletters. Vul de uitwisseling in die je hoort en druk Enter: jouw uitwisseling gaat de lucht in en het QSO wordt gelogd.

**Volgnummer.** Het nummer staat naast de invoerregel. Het gaat pas omhoog als je logt. Zend je het nummer twee keer omdat de ander AGN vraagt, dan blijft het hetzelfde. Verlaat je een station waaraan je een nummer zond zonder te loggen, dan zegt een waarschuwing welk nummer dat was.

**Dupe.** Een rood DUPE betekent dat je dit station op deze band in deze wedstrijd al werkte. Je mag toch loggen.

**Nummer testen zonder dat het telt.** Klik op Test (oranje). QSO's gaan nu naar een apart testlogboek met een eigen teller vanaf 001. Je echte nummer en logboek veranderen niet. Met Testlog wissen verwijder je ze weer. Test QSO's komen nooit in een export van het echte logboek.

**Gegevens van het andere station.** Klik in de ontvangen tekst op een woord en kies waar het heen moet: roepletters, rapport, nummer, extra veld, naam of QTH. Na je uitwisseling stelt CWLD zelf voor wat het in het antwoord hoorde (grijs). Druk Tab in een leeg veld om het over te nemen.

**Nieuwe wedstrijd.** Zet contest uit en weer aan. CWLD vraagt Doorgaan of Opnieuw. Opnieuw begint bij 001 en eerdere QSO's tellen niet meer als dupe.

**Toetsen.** F1 tot F8 zenden. Enter doet het volgende logische ding. Ctrl+R wisselt Run en S&P. Ctrl+W maakt de invoerregel leeg. Esc stopt het zenden. Ctrl+F6 is Sterkste en Ctrl+F7 is AFC (buiten contest mode F6 en F7).

**Logboek en export.** Onder de audio-scope staan de laatste 5 QSO's. Klik Verbeter om er een aan te passen, of × om hem te verwijderen. Klik Exporteren voor een ADIF-bestand: kies wat (het echte logboek, de lopende wedstrijd of het testlogboek), van en tot (UTC) en de band. Het bestand komt in de map Documenten, submap CWLD. In Log4OM importeer je het via het menu Bestand, ADIF importeren. Het veld CONTEST_ID moet je eerst controleren in de officiële ADIF-lijst.

**Gewone QSO's.** Zonder contest mode staat in het QSO-paneel een knop Loggen, met een veld voor het ontvangen rapport.

## 18. Stationsprofielen en activaties (0.8.0)

**Profiel kiezen.** Bovenaan, naast de frequentie, staat een keuzelijst met je profielen (Thuis en Portable). Tijdens een activatie is hij vergrendeld, zodat je niet per ongeluk wisselt. Profielen bewerk je via Instellingen, Station: naam, achtervoegsel (bijvoorbeeld /P), grid, vermogen, radio, antenne en opmerking.

**Een activatie starten.** Klik in het paneel Activatie op Nieuwe activatie. Vul de WWFF/ONFF-referentie in (bijvoorbeeld ONFF-0599, mag leeg), het grid (zes tekens, verplicht als je portable bent) en het vermogen. Klik Activatie starten. Elk QSO krijgt nu referentie, grid en vermogen mee.

**Tijdens de activatie.** Het paneel toont het grid in grote letters, het aantal QSO's en unieke roepletters. Gaat de activatie over middernacht UTC, dan staat er een waarschuwing. Het doel van 44 QSO's staat als "na te kijken" tot het bevestigd is. Voor park naar park vul je in het QSO-paneel Hun referentie in.

**Exporteren.** Logpaneel, Exporteren. Kies Huidige activatie, Alleen QSO's die nog niet geëxporteerd zijn, Alle QSO's, een contest of het testlog. Klik eerst Controleren. Fouten blokkeren de export, waarschuwingen niet. Controleer het grid in grote letters. Het bestand krijgt voor WWFF een naam zoals `ON3VZ-P @ ONFF-0599 20261103.adi` (de schuine streep wordt een streepje).

**Hoogte van de bandscoop.** Instellingen, Weergave: klein, midden of groot, apart voor gewoon gebruik en voor contestmodus.

## 19. Oefenen tegen gesimuleerde stations (0.9.0)

**Wat het is.** In OEFENEN zendt CWLD nooit. Start je een oefening, dan luistert de decoder naar een oefenband met gesimuleerde stations in plaats van naar de radio. Je hoort die stations via de computer (koptelefoon of luidsprekers). Draai de AF-knop van de radio dicht, anders hoor je de band erdoorheen.

**Starten.** Klik bovenaan rechts op OEFENEN. Rechts verschijnt het paneel Oefenen. Kies de oefening:
- QSO: een station roept CQ. Wacht tot je de CQ hoort en antwoord met de chips of door te typen. Het station geeft rapport, naam en QTH. Antwoord en log het QSO.
- QSO: jij roept CQ. Zend CQ (chip of typen). Een station roept je aan. Zend zijn roepletters met je rapport, hij antwoordt met zijn gegevens.
- Contest (Run). Kies een wedstrijdprofiel in het paneel. Na Starten staat de contestregel klaar in testmodus. Druk F1 of Enter voor CQ en werk de aanroepers af met F2, F3 en Enter zoals in een echte wedstrijd.

Stel snelheid (van en tot), sterkte, bandruis, fading, het aantal aanroepers en de duur in. Klik Oefening starten.

**Wat de stations doen.** Ze reageren op wat je zendt. AGN of ? laat herhalen. NR? geeft het nummer opnieuw. Een deel van een roepletters met een vraagteken (bijvoorbeeld PA3?) laat de passende stations opnieuw roepen. Een verkeerde roepletters wordt verbeterd. Wie te lang moet wachten, roept opnieuw en vertrekt na een tijd.

**Hulp van de decoder.** Vink Hulp van de decoder uit om op het gehoor te oefenen. De gedecodeerde tekst wordt dan verborgen (puntjes) en niets wordt voor je ingevuld.

**Score.** Klik Stoppen en score tonen (of wacht tot de tijd om is). Je ziet het aantal juiste QSO's, verkeerde roepletters met de echte roepletters ernaast, verkeerde uitwisselingen, afgewerkte maar niet gelogde stations en je rate. De QSO's staan in het testlog en tellen nooit mee in het echte logboek.

**Morse leren.** De link Morse leren in het paneel opent de oefen-app voor letters en gehoortraining.

## 20. Begeleide eerste start (0.10.0)

Bij de eerste start opent een venster met vier stappen. Je kunt het altijd overslaan en later opnieuw openen via Instellingen, Updates en info, Begeleide start. Het zendt nooit iets.

1. **Radio zoeken.** Kies het type radio. CWLD kiest zelf de poort als er maar één op jouw radio lijkt (bij de IC-7300 MK II de CI-V-poort). Vindt het niets, controleer dan kabel, radio aan en driver, en kijk in Apparaatbeheer: rechtsklik op de Start-knop, kies Apparaatbeheer, open Poorten (COM en LPT). Klik op Test de verbinding.
2. **Radio controleren.** Zet in de radio CI-V Transceive aan: MENU, SET, Connectors, CI-V. Draai aan de afstemknop: volgt de frequentie in het venster, dan staat het goed. CWLD kan deze instelling niet uit de radio lezen. Bij de QMX hoort hier niets, wel in stap 3 de zijtoon.
3. **Geluid kiezen.** Kies de ingang van de radio (USB Audio CODEC), klik op Begin te luisteren en kijk naar de niveaubalk. Blijft die leeg: controleer de microfoon-privacy van Windows en zet geluidsverbeteringen uit. QMX: zet in het QMX-menu Audio, Sidetone volume hoog en Sidetone abs/rel op Relative.
4. **Station.** Roepletters, naam, QTH en locator. De stille test toont de CQ die CWLD zou zenden en hoe lang die duurt, zonder iets te zenden.

## 21. Meer ruimte (0.10.2)

- De activatie zit niet meer in het rechterpaneel maar als groene knop rechts in de tabbalk. Zonder activatie heet ze Activatie en opent ze het formulier. Tijdens een activatie toont ze referentie en aantal QSO's. Klik erop voor grid, tellers, doel en Activatie beëindigen.
- In het audiopaneel staan de decoderknoppen achter de schakelaar Afstellen. De sneltoetsen F6 (Sterkste) en F7 (AFC) werken ook als de knoppen verborgen zijn.
- Het QSO-paneel toont de uitleg bij een leeg roepletterveld als tooltip.
