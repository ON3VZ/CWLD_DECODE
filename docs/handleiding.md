# CWLD, korte handleiding (testversie, bijgewerkt voor 0.20.6)

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

Sinds 0.20.6 staat bij Instellingen, **Updates en info**, ook een knop **Helppagina openen**. Die opent de helppagina van de website in je standaardbrowser, in de taal van de app. Je hebt internet nodig.

## 9. Bandscoop

Bovenaan staat de bandscoop van de radio: hetzelfde spectrum als op het scherm van de IC-7300 MK II, met een lijn (sterkte) en een waterfall (verloop in de tijd). De rode lijn is je VFO. Beweeg met de muis over de scoop om de frequentie te zien, klik op een signaal en de radio stemt erheen.

- Hoe breed de scoop is, kies je op de radio zelf (de SPAN-knop op het scoopscherm, bv. ±25 kHz). CWLD volgt dat.
- CWLD zet de scoopgegevens zelf aan als je de radio verbindt. Op de radio moet niets extra ingesteld worden, behalve wat in de eerste start staat (CI-V via USB).
- Rechts, onder het QSO-blok, staat het blok **Audio**: wat de decoder hoort binnen het filter van de radio. Een klik daar zet de decoder op die toon.
- Beide kun je verbergen: klik onder de bandscoop op **Bandscoop verbergen**, of in het blok Audio op **Verbergen**. Je kunt ook naar Instellingen, **Weergave en taal** gaan. Een verborgen deel haal je terug met de link bovenaan het scherm.
- De QMX heeft geen bandscoop; daar zie je alleen de audio-waterfall.
- Sinds 0.18.2 is de scoop van boven naar onder opgebouwd uit: een strook voor de spotlabels (zo hoog als nodig), het spectrum, de frequentieschaal en de waterfall. Het spectrum is een gladde lijn met een zachte vulling en een flauwe lijn voor de pieken. Het bereik past zich aan de band aan: de ruisvloer zit onderaan en het sterkste signaal bovenaan, zodat zwakke signalen op een brede span (bv. 200 kHz) niet in een vlakke lijn verdwijnen.
- Sinds 0.20.2 hebben de labels een lijntje met een klein pijltje naar hun exacte frequentie. Is de plaats van een label bezet, dan schuift het een stukje opzij (hoogstens 64 pixels) of een rij lager in plaats van te verdwijnen. Zie hoofdstuk 41 voor split.

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
- Smal, Breed of Steil: Breed heeft dubbele bandbreedte, voor driftende of tjirpende signalen. **Steil** (sinds 0.20.0) heeft de breedte van Smal maar veel steilere flanken: een station 60 Hz verderop is zo'n 40 dB zwakker dan bij Smal. Kies Steil als een sterk station vlak naast het jouwe de decoder stoort. Op heel zwakke signalen zonder buur blijft Smal iets beter, daarom staat Smal standaard. De twee rode lijnen in het waterfall tonen het filter.
- Letters samenvoegen (Instellingen, Decoder, standaard aan): seint iemand een roepletter of naam met een woordpauze na elke letter, dan toont CWLD D L 4 S A V als DL4SAV en H A N S als HANS. Een K of R vlak na een uitgespelde roepletter blijft los (DL4SAV K). Losse letters die alleen horen te staan (R R, E E, K) blijven zoals ze zijn. Letters wachten tot het volgende woord of tot een pauze van 20 eenheden.
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

## 22. Stop, noodstop en foutteken (0.10.4)

- **Esc of de knop Stop:** het zenden stopt meteen en de wachtrij wordt gewist.
- **Noodstop:** twee keer snel Esc (binnen twee seconden) of de knop Stop + QRX. CWLD stopt en zendt QRX QRX, zodat het andere station wacht terwijl jij je fout oplost. De tweede Esc telt alleen als de eerste een zending stopte, dus op een stille radio zendt Esc nooit iets.
- **Fout:** de knop Fout stopt wat verzonden wordt en zendt het foutteken (acht puntjes). Daarna zend je het juiste woord. De tekst in je zendvak blijft staan.
- In LUISTEREN wordt nooit iets gezonden. In OEFENEN worden QRX en het foutteken alleen gesimuleerd.

## 23. Logkoppeling met Log4OM en N1MM (0.11.0)

CWLD kan elk QSO dat je logt via UDP doorgeven aan een logboekprogramma. Alles staat standaard uit. Test-QSO's worden nooit verstuurd. Een mislukte verzending houdt het loggen in CWLD niet tegen.

### Log4OM instellen (ADIF)

Eerst in Log4OM, dan in CWLD. De Log4OM-stappen zijn gebaseerd op een beschrijving van een ander programma (VarAC) en op de Log4OM-handleiding; de menunamen kunnen in jouw versie licht verschillen.

1. Klik in Log4OM op het tandwiel Settings (instellingen) en kies Program Configuration.
2. Open Software integration, dan Connections, dan UDP.
3. Zoek het deel UDP INBOUND. Vul bij poort 2333 in en geef de verbinding een naam, bijvoorbeeld CWLD.
4. Kies bij Service Type de waarde **ADIF_MESSAGE**.
5. Klik op de plusknop (+) om de verbinding toe te voegen en zet een vinkje bij de verbinding (de V).
6. Klik op Save and Apply.
7. Ga in CWLD naar Instellingen, Logkoppeling, zet "Elk gelogd QSO naar Log4OM sturen" aan, laat het adres op 127.0.0.1 en de poort op 2333 staan, en klik Opslaan.
8. Log een echt QSO in CWLD. Het moet in Log4OM verschijnen. Of klik op Laatste QSO opnieuw sturen: Log4OM meldt dan dat het al bestaat, dat is normaal.

Let op:

- **Eén lezer per poort.** Maar één programma kan op een UDP-poort luisteren. Gebruikt WSJT-X of een ander programma poort 2333 al, kies dan een andere poort in beide programma's.
- **Maak geen JT-verbinding aan** voor CWLD, alleen ADIF_MESSAGE. Anders komt het QSO twee keer aan.
- **Eén logweg tegelijk.** Gebruik niet tegelijk de koppeling en een ADIF-import van hetzelfde QSO.
- **Draait Log4OM op een andere pc?** Vul dan bij Adres het IP-adres van die pc in (open op die pc een opdrachtprompt en typ ipconfig).
- **[TEST]** Dat Log4OM één ruw ADIF-record per bericht verwerkt, is uit de handleiding en een derde bron afgeleid maar nog niet op jouw pc bevestigd. Zie test A39.

### N1MM-formaat

N1MM Logger+ kan **geen QSO's via UDP ontvangen**. Het zendt alleen zelf berichten uit (contactinfo, contactreplace, contactdelete, RadioInfo) en ontvangt via UDP alleen spectrumgegevens en een frequentiecommando. CWLD kan dus niet in N1MM loggen via deze weg.

De N1MM-koppeling in CWLD zendt wel hetzelfde berichtformaat (contactinfo) uit voor programma's die naar zulke uitzendingen luisteren, bijvoorbeeld een scorebord of een tracker. Zet "QSO's ook uitzenden als N1MM Logger+-berichten" aan en vul bij Bestemmingen één of meer adressen in als adres:poort, gescheiden door een spatie (standaard 127.0.0.1:12060, de standaardpoort van N1MM).

Bronnen: [Log4OM, Working with JT](https://www.log4om.com/l4ong/usermanual/Working_with_JT.pdf), [VarAC en logboekprogramma's](https://www.varac-hamradio.com/post/integrating-varac-with-your-qso-logging-application), [N1MM, externe UDP-uitzendingen](https://n1mmwp.hamdocs.com/appendices/external-udp-broadcasts/).

## 24. Paddle oefenen en een stille radio (0.12.0, aangepast in 0.13.1)

In OEFENEN kun je met de paddle op de radio seinen, zonder dat er iets gezonden wordt. CWLD hoort je zijtoon, leest wat je seint en meet je spatiëring.

### Radio stil tijdens het oefenen

Staat "Radio stil tijdens het oefenen" aan (in het oefenpaneel rechts, standaard aan), dan doet CWLD bij de overstap naar OEFENEN twee dingen op de IC-7300 MK II:

- De squelch gaat dicht. De band valt stil in je hoofdtelefoon.
- Break-in gaat uit. De paddle maakt dan alleen de zijtoon, de radio zendt niet.

Het AF-volume blijft staan. Op de IC-7300 MK II bepaalt de AF-knop ook hoe luid de zijtoon op de USB-audio staat: staat hij op 0, dan hoort CWLD je paddle niet. Gemeten door ON3VZ: met AF op ongeveer 20 procent en de squelch dicht is de band stil en herkent CWLD de paddle. Tot en met 0.13.0 zette CWLD het AF-volume op 0, en dan werkte paddle oefenen niet.

Ga je naar LUISTEREN of LIVE, of sluit je CWLD, dan zet CWLD squelch en break-in terug zoals ze waren. Draai je tijdens het oefenen zelf aan de SQL-knop, dan laat CWLD dat zo. Valt de pc of CWLD onverwacht uit, dan blijft de squelch dicht: draai dan de SQL-knop linksom en zet Break-in weer aan met de knop BK-IN op de radio.

Onder het vakje staat wat de radio meldt: volume, squelch en break-in. Staat er "niet gemeld", dan antwoordde de radio niet op die vraag en doet CWLD niets: draai dan zelf de SQL-knop rechtsom.

**[TEST]** De commando's zijn die van de IC-7300 (squelch `14 03`, break-in `16 47`, AF-niveau `14 01` alleen gelezen). Dat ze op de MK II gelijk zijn, is nog niet bevestigd (test A40). Bij de QMX doet CWLD dit niet.

### Hoort CWLD je zijtoon?

De zijtoon moet op de USB-audio van de radio staan, anders hoort CWLD niets. Standaard stuurt de radio die niet mee. Zet dit eenmalig op de radio (gevonden en bevestigd door ON3VZ op zijn IC-7300 MK II):

1. Druk op MENU en tik op SET.
2. Kies Connectors, dan USB AF/IF Output.
3. Zet "AF Beep/Speech... Output" op ON.
4. Laat het AF-volume van de radio op ongeveer 20 procent staan (niet op 0, zie hierboven).

Deze instelling stuurt ook de pieptonen en spraakmeldingen van de radio mee op de USB-audio. Voor het zenden maakt dat niets uit. Gebruik je ook WSJT-X of andere digitale software, dan hoort die die tonen ook; zet de instelling dan eventueel weer uit.

Zo kijk je na of het werkt:

1. Zet CWLD in OEFENEN en start het luisteren met de audio-ingang van de radio (Instellingen, Audio, kies "USB Audio CODEC", klik Toepassen).
2. Klik rechts op "Start paddle oefenen" en sein een paar tekens.
3. Verschijnt er tekst onder "Gehoord", dan werkt het. Blijft de melding "Nog geen zijtoon gehoord" staan:
   - Controleer de instelling "AF Beep/Speech... Output" hierboven en of het AF-volume niet op 0 staat (de paddle-kaart waarschuwt dan ook).
   - Zet op de radio het zijtoonniveau hoger: druk op MENU, kies KEYER, dan EDIT/SET, dan CW-KEY SET, en zet "Side Tone Level" op ongeveer 70 procent.
   - Zet de Monitor-functie van de radio aan (zie de Basic-handleiding van de radio bij Monitor) en probeer opnieuw.
   - Lukt het nog niet, meld het dan. Dan bouwen we de terugvalweg (CWLD maakt zelf de toon).

### Een oefening

1. Kies de oefening: tekengroepen, je zwakke tekens, woorden en afkortingen, roepletters of QSO-zinnen.
2. Klik "Start paddle oefenen". In het midden verschijnt de tekst in grote letters.
3. Sein de tekst. Wat CWLD hoort, staat eronder. Elke oefening (ook de simulator) start in een eigen tabblad "Oefenen", zodat ze nooit in een echt QSO terechtkomt.
4. Ben je klaar en stop je twee seconden, dan beoordeelt CWLD vanzelf. Je kunt ook op "Klaar" klikken.
5. Je ziet welke tekens fout waren (rood, met wat CWLD in de plaats hoorde), je score, het aantal juiste tekens, je snelheid en je spatiëring met tips. Klik "Volgende" of "Opnieuw".

### Wat de score meet

CWLD leest wat je seinde rechtstreeks uit je sleuteltijden (wanneer de zijtoon aan en uit ging), met je eigen snelheid gemeten over de hele poging. Dat is betrouwbaarder dan de gewone decoder, die eerst een paar tekens nodig heeft om je snelheid te vinden. Tijdens paddle oefenen staat de spraakcontrole van de decoder uit: een zijtoon met boventonen werd anders voor spraak aangezien en dan kwam er niets door (gevonden met ON3VZ, hersteld in 0.13.2).

- **Juist**: het aandeel tekens dat goed overkwam. Een pauze midden in een teken telt als fout, want een keyer knipt dan het teken in twee (A wordt E T).
- **Spatiëring**: de pauze in een teken (ideaal 1 punt), tussen tekens (3 punten) en tussen woorden (7 punten), en bij een seinsleutel ook de streeplengte (3 punten) en hoe gelijk je punten en strepen zijn.
- Met de keyer van de radio zijn punten en strepen altijd perfect. Dan is alleen je spatiëring van jou, en CWLD zegt dat ook.
- De score van een oefening is voor 70 procent het aantal juiste tekens en voor 30 procent de spatiëring.

CWLD onthoudt per teken hoe vaak het goed en fout ging. De oefening "Mijn zwakke tekens" vraagt de tekens die het vaakst fout gingen, en in de andere oefeningen komen ze ook vaker terug. Rechts staan je laatste vijf resultaten (tijd in UTC). "Mijn resultaten vergeten" wist ze.

## 25. Geen tekst bij SSB of spraak (0.12.0)

- Staat de radio op USB, LSB, AM, FM of RTTY, dan toont CWLD geen gedecodeerde tekst. Boven het gesprek staat dan "Decoder gepauzeerd: de radio staat op USB". Zet de radio op CW en het decoderen start weer, met een schone lei.
- Ook op CW kijkt CWLD of het geluid op spraak lijkt (een SSB-station op de verkeerde plaats). Dan staat er "Spraak of een breed signaal, geen CW-toon: de decoder wacht". Zodra de spraak stopt, start het decoderen na ongeveer anderhalve seconde vanzelf.
- In OEFENEN geldt de modusregel niet, want dan komt het geluid van de oefenband.
- Gemeten op de testbank: uit synthetische spraak kwamen voorheen 36 tot 54 tekens per minuut, nu 0 tot 5 bij een sterk of matig signaal. De CW-resultaten bleven gelijk. Details in docs/meetresultaten_decoder.md.

## 26. Naam en QTH opzoeken met QRZ.com (0.13.0)

Heb je een QRZ.com account, dan kan CWLD de voornaam en woonplaats van het station in het QSO-paneel opzoeken en invullen.

### Instellen

1. Klik op het tandwiel, en kies links "Logkoppeling".
2. Scrol naar "QRZ.com opzoeken" en vink "QRZ.com gebruiken om roepletters op te zoeken" aan.
3. Vul je QRZ.com gebruikersnaam en wachtwoord in en klik "Opslaan en testen".
4. Staat er "Inloggen werkt", dan is het klaar. Heb je het XML-abonnement (XML Logbook Data) niet, dan zegt CWLD dat QRZ.com de naam en plaats mogelijk weglaat.

### Wat gebeurt er

- Zodra een roepletterreeks in het paneel staat en je even niet meer typt, vraagt CWLD die ene roepletterreeks aan QRZ.com via HTTPS.
- Is "Naam" of "QTH" leeg, of vulde CWLD ze zelf in voor een vorig station, dan vult CWLD ze in. Wat je zelf typte, blijft staan. Onder de velden staat wat QRZ.com antwoordde, alleen zolang die roepletters in het paneel staan.
- Antwoorden worden een dag onthouden, zodat QRZ.com niet onnodig bevraagd wordt.
- Is QRZ.com onbereikbaar of klopt het wachtwoord niet, dan komt er één rustige melding en gaat loggen gewoon door.

### Waar blijft je wachtwoord

Het wachtwoord staat versleuteld in Windows Referentiebeheer, voor je eigen Windows-account. Het staat nooit in het instellingenbestand, in het logboek of op het scherm. Wil je het wissen, klik dan "Opgeslagen wachtwoord wissen". Op Linux gebruikt CWLD de Secret Service (bijvoorbeeld GNOME Keyring of KWallet); is die er niet, dan meldt CWLD dat het wachtwoord niet bewaard kan worden.

## 27. De Koch-methode: tekens kiezen en kopiëren (0.14.0)

De Koch-methode leert je morse door te beginnen met maar twee of drie tekens op volle snelheid. Pas als je die goed kopieert, komt er een teken bij. Zo leer je de klank van elk teken en tel je geen punten en strepen.

### Je tekens kiezen

1. Zet de radio in OEFENEN (het kopje bovenaan). Rechts staat de kaart "Tekens leren (Koch)".
2. Sleep het schuifje: "De eerste 8 van 40 tekens" gebruikt de eerste acht van de Koch-volgorde (K M R S U A P T).
3. Wil je een teken ook gebruiken dat nog niet aan de beurt is, of juist weglaten? Klik op dat teken. Gekleurd betekent: in gebruik. Minstens twee tekens blijven staan.
4. Het omrande teken is het nieuwste. Dat komt vaker terug, net als tekens die je eerder fout deed.

De keuze wordt onthouden. Standaard staan alle tekens aan, dus er verandert niets tot je ze beperkt.

### Seinen met de paddle met jouw tekens

De paddle-oefeningen (kaart "Paddle oefenen") gebruiken alleen je gekozen tekens. Woorden, roepletters en zinnen komen alleen voor als ze volledig uit jouw tekens bestaan. Zijn het er te weinig, dan staat er een korte melding en krijg je tekengroepen. Roepletters hebben minstens drie letters en een cijfer nodig.

### Luisteren en typen

1. Kies bij "Wat wordt afgespeeld" groepen van 5 tekens of woorden.
2. Stel de tekensnelheid in (standaard 20 WPM) en de totale snelheid (standaard 12 WPM). Het verschil zit in de pauzes: de tekens blijven snel, de ruimte ertussen wordt langer (Farnsworth). Werk langzaam omhoog met de totale snelheid, niet met de tekensnelheid.
3. Klik "Start luisteren". Het geluid komt van deze computer, nooit via de radio, dus je kunt dit ook zonder verbonden radio doen.
4. Typ wat je hoort in het vak en druk op Enter. Fout getypte tekens worden rood onderstreept met wat jij typte eronder. "Nog eens afspelen" speelt dezelfde tekst opnieuw. Esc stopt.

### Wanneer komt er een teken bij?

CWLD telt de laatste drie rondes met dezelfde tekens. Zijn ze samen minstens 90 procent juist (en minstens 30 tekens), dan verschijnt een voorstel om het volgende Koch-teken toe te voegen. Je klikt zelf op "Voeg ... toe"; CWLD doet het nooit vanzelf.

Onder de kaart staan de tekens waar je het vaakst fout op zat en je laatste vijf rondes. "Vergeet mijn resultaten" wist ze.

## 28. Slordige morse en woordgrenzen (0.15.0)

Niet iedereen seint volgens het boekje. De een spelt zijn naam met de letters ver uit elkaar, de ander plakt CQ en DE aan elkaar, een derde laat lange pauzes binnen een teken. CWLD leert daarom per zender hoe hij spatieert, uit zijn eerste tekens, en beslist daarmee waar een woord eindigt.

Wat je daarvan merkt:

- **Definitieve tekst komt iets later.** CWLD houdt ongeveer vijf tekens vast om een pauze te kunnen beoordelen met wat erna komt. Je ziet die tekens wel meteen, in de lichtere voorlopige tekst. Na een stilte van ruim een woordpauze wordt alles definitief.
- **Bekende woorden helpen.** Twijfelt de timing, dan kiest CWLD de lezing die een gewoon CW-woord geeft (CQ, DE, 599, UR, TNX), of een roepletterreeks die net voorbijkwam. Is de timing duidelijk, dan wint altijd de timing.
- **Nieuwe zender, nieuwe stijl.** Bij een andere toon of na een lange stilte begint het leren opnieuw.

Wat nog fout kan gaan: een zender die midden in een uitzending van tempo wisselt (traag spellen en daarna snel), en een punt of streep die in een fade verdwijnt. Stuur in zo'n geval een opname (knop OPNAME) met wat er volgens jou gezonden werd.

Bij paddle oefenen (hoofdstuk 24) hoeft de decoder niet meer op je zijtoon te staan: CWLD zoekt de zijtoon zelf tussen 300 en 1200 Hz.

## 29. Fading (QSB): Doorlezen of Streng (0.16.1)

In de kaart Audio op het hoofdscherm, onder de piekmeter, staat de schakelaar "Fading (QSB)". Je hoeft er de instellingen niet voor te openen. Onder de schakelaar staat altijd in één regel wat de gekozen stand doet.

| Stand | Wat het doet | Nadeel |
|---|---|---|
| **Doorlezen** (standaard) | Punten en strepen die in een fade tot 12 dB zwakker worden tellen mee, zolang ze op dezelfde toon zitten. | Iets meer foute tekens bij een heel zwak signaal (rond de ruis) en bij heel ongelijk handwerk. |
| **Streng** | Alleen punten en strepen op bijna volle sterkte tellen. | Bij QSB verliezen tekens punten en breken woorden (FAILED wordt FAIA ED). |

Wanneer welke stand:

- Gaat het signaal hoorbaar op en neer en mis je stukken van woorden: **Doorlezen**.
- Is het signaal stabiel maar heel zwak, of zie je extra punten en foute letters die er niet zijn: **Streng**.

De keuze werkt meteen en wordt onthouden. Beweeg de muis over een stand voor de volledige uitleg.

## 30. DX-cluster: spots op de bandscoop (0.18.0)

Een DX-cluster is een dienst waar stations en skimmers (ontvangers die CW zelf decoderen) melden welk station op welke frequentie in de lucht is. CWLD leest die spots en zet ze als labels op de bandscoop. Je hebt internet nodig. CWLD **leest alleen**: er wordt nooit een spot of bericht geplaatst.

**Aanzetten**

1. Klik rechtsboven op het tandwiel (Instellingen). Kies links **Station** en vul bij "Je roepletters" je roepletters in. De cluster heeft ze nodig om aan te melden. Er is geen ingebouwde roepletter.
2. Kies links **DX-cluster**.
3. Laat "Cluster" op **Automatisch** staan. CWLD probeert dan ON0DXK (België) eerst en gaat naar de volgende als die niet antwoordt.
4. Klik op **Verbinden**. Na een paar seconden staat er "Verbonden met ... (software). ... spots."
5. Wil je dat CWLD dit bij elk opstarten doet, vink dan "Verbinden met de cluster als CWLD start" aan en klik **OK**. Standaard staat dit uit.

Je kunt ook zonder instellingen verbinden: onder de bandscoop staat de knop **Clusterspots tonen**. Dat gebruikt de instellingen zoals ze zijn.

**De labels lezen**

De labels zijn rustige vakjes met de roepletters in lichtgrijs. Alleen het bolletje links heeft een kleur (sinds 0.18.1):

| Bolletje | Betekenis |
|---|---|
| Groen | Het station staat niet in je logboek |
| Oranje | Je werkte het op een andere band |
| Grijs, label vervaagd | Al in je logboek op deze band. In contestmodus: een dupe in de lopende contest |

- Klik op een label: de radio stemt af op die frequentie. Er wordt niets gezonden.
- Houd de muis op een label voor de details: wie spotte het, sterkte, snelheid, hoe lang geleden.
- Staan er te veel labels op elkaar, dan blijven de stations die nog niet in je logboek staan zichtbaar en staat in de balk "... verborgen". Maak de bandscoop breder of groter (Instellingen, Weergave) of wacht tot oude spots verdwijnen.
- **Spots verbergen** in de balk onder de scoop zet de labels uit zonder te verbreken. **Verbreken** staat in Instellingen, DX-cluster.
- Nadat je een QSO logt, vervaagt het label van dat station meteen (of het bolletje wordt oranje voor een andere band).

**Skimmerspots**

Met "Ook spots van skimmers tonen" aan zie je veel meer stations, volgens een schatting zo'n twintig keer zoveel, maar ook enkele foute roepletters die een skimmer verkeerd las. Zet het uit als je alleen spots van mensen wilt.

**Welke cluster?**

Klik op **Test de clusters**. CWLD maakt even verbinding met elke node van de lijst, zonder aan te melden, en toont welke antwoorden en hoe snel. Kies daarna er één, of laat het op Automatisch. Onder "Lijst met clusters" kun je er zelf een toevoegen (naam, adres, poort) of de standaardlijst herstellen.

**Commando's typen (zoals in N1MM)**

Open onderaan **Clustervenster (voor commando's)**. Hier zie je wat de node nog meer zegt (welkomsttekst, antwoorden). Typ een commando en klik **Verzenden**, bijvoorbeeld `sh/dx 10` voor de laatste tien spots, of `set/skimmer` om skimmerspots te vragen. Alleen commando's die lezen of je eigen filter instellen worden doorgelaten. Commando's die iets zouden plaatsen (`dx`, `announce`, `talk`, `wwv`, `wx`) worden geblokkeerd.

**Wat het niet is**

Een spot betekent dat een station ergens gehoord is, niet dat jij het nu hoort. De frequentie is ongeveer op 0,1 kHz nauwkeurig en spots lopen soms een paar minuten achter. Zonder internet zie je geen spots.

**Spots helpen de decoder (0.18.1)**

Staat er een spot binnen 700 Hz van je frequentie, dan krijgt de decoder die roepletters als hint. Hij gebruikt een hint alleen waar hij zelf twijfelt, bijvoorbeeld als een punt of streep in een fade wegviel. Je hoeft er niets voor in te stellen: het werkt zolang de cluster verbonden is, ook als de labels verborgen zijn. Stem je weg, dan vervallen de hints. Op de testbank kwam een roepletter met weggevallen punten vaker goed door (60 naar 85 procent), en een foute spot veranderde de tekst niet. Op de band is dit nog niet gemeten.

## 31. Waarschuwing: een gat in het geluid (0.18.4)

Staat er in de kaart Audio een oranje melding "Een gat in het geluid rond ... Hz", dan is het geluid van de radio in het midden veel stiller dan aan beide kanten. Dat doet een notchfilter. Het gevolg: een station dat je netjes op je CW-toon afstemt wordt weggefilterd, en je hoort alleen stations die er een paar honderd Hz naast zitten.

Zo los je het op:

1. Kijk op het scherm van de radio of er "MN" (handmatige notch) of "AN" (automatische notch) staat.
2. Druk op de knop NOTCH tot die aanduiding verdwijnt.
3. Staat er geen notch aan, zet dan Twin PBT in het midden: houd PBT-CLR één seconde ingedrukt.
4. De melding in CWLD verdwijnt vanzelf na een paar seconden.

CWLD schakelt de notch zelf nooit. De melding komt alleen bij een gat van minstens 20 dB dat drie seconden blijft.

## 32. Het logboekscherm (0.19.0)

Klik bovenaan op **Logboek** (naast het tandwiel), of in de kaart Logboek op **Alle QSO's**.

**Zoeken en filteren.** Typ in het zoekvak een stuk van roepletters, naam, QTH, opmerking of referentie. Daarnaast kies je datums (in UTC), band, contest en activatie. Met "Nog niet geëxporteerd" zie je wat nog naar Log4OM moet, met "Testlogboek" het aparte testlogboek. "Filters wissen" zet alles terug.

**Kolommen.** Klik op **Kolommen** en vink aan wat je wilt zien, bijvoorbeeld vermogen, rig en antenne voor portabel werk. CWLD onthoudt je keuze. Klik op een kolomkop om te sorteren, nog eens om de volgorde om te draaien.

**Exporteren van wat je ziet.** De knop "Deze N QSO's exporteren (ADIF)" schrijft precies de QSO's van je filter naar een bestand in de map CWLD in Documenten. Eerst loopt dezelfde controle als in het exportvenster. Daarna verschijnen het pad en de knop **Map openen**. In Log4OM lees je het bestand in via Bestand, ADIF importeren.

**Wijzigen.** Klik op een rij om het QSO te wijzigen of te verwijderen. Verwijderen markeert het QSO alleen, er gaat niets verloren.

## 33. Notch en Twin PBT vanuit CWLD (0.19.0)

CWLD leest van de radio of de handmatige notch (MN) of de automatische notch (AN) aanstaat, en of Twin PBT verdraaid is. Is dat zo, of hoort CWLD een gat in het geluid, dan staat er een oranje melding in de kaart Audio met twee knoppen:

- **Notch uit** zet MN en AN uit.
- **PBT in het midden** zet beide PBT-knoppen terug in het midden.

CWLD zet deze nooit zelf om. Gebruik je de notch bewust tegen een draaggolf, dan blijft hij staan tot je klikt.

## 34. Een QSO verbeteren (0.19.1)

Klik in het logboek op een rij (of op "Verbeter" in de logkaart). Alle velden zijn te wijzigen: datum en tijd in UTC, frequentie in kHz, mode, rapporten, uitwisseling, naam, QTH, referentie van de ander, opmerking, en onder **Mijn station** je eigen gegevens.

Wijzig je de frequentie, dan vraagt CWLD eerst om bevestiging. Het toont de oude en nieuwe frequentie en of de band verandert. Klik op **Ja, wijzigen** om te bewaren of op **Terug** om verder te bewerken. De band volgt altijd de frequentie. Ligt de frequentie buiten de amateurbanden, dan blijft de band zoals hij was.

Een verbeterd QSO staat daarna weer als "nog niet geëxporteerd".

## 35. Logboek als CSV (0.19.2)

Klik in het logboek op **CSV**. CWLD schrijft de QSO's zoals ze nu getoond worden naar een CSV-bestand in de CWLD-map, met de kolommen die je via **Kolommen** koos en in dezelfde volgorde. Klik daarna op **Map openen** en dubbelklik het bestand: Excel opent het met de juiste tekens en kolommen. Het scheidingsteken is een puntkomma. Een CSV-export markeert niets als geëxporteerd, want dat doet alleen de ADIF-export.

## 36. Rustiger scherm en minder tabbladen (0.19.3)

Scrol je in het gespreksvenster omhoog om iets terug te lezen, dan blijft het beeld staan als er nieuwe tekst bijkomt. Sta je helemaal onderaan, dan loopt het venster vanzelf mee. De waarschuwing over de notch of Twin PBT staat nu over het audiospectrum, zodat de rest van de kaart Audio niet meer verspringt.

Het maximum aantal open tabbladen stel je in via **Instellingen**, onderdeel met de tabbladen, bij **Maximaal aantal open tabbladen** (standaard 5, 0 is geen limiet). Het oudste rustige gesprek sluit eerst. Het gesprek waarin je zit en gesprekken met gezonden tekst die nog niet gelogd zijn sluiten nooit vanzelf.

## 37. Terugspoelen: de laatste seconden opnieuw lezen (0.20.0)

CWLD bewaart de laatste minuut geluid. Heb je iets gemist, dan lees je het opnieuw, ook op een andere toon.

1. Zoek rechts de kaart **Audio** en klik op **Terugspoelen** (het pijltje klapt open).
2. Kies hoe ver terug: **10 s**, **20 s**, **30 s** of **60 s**.
3. Kies hoe CWLD het leest:
   - **Zelfde toon**: de toon waarop de decoder nu luistert.
   - **Sterkste**: de sterkste getoetste toon in dat stuk.
   - Een van de toonhoogtes onder **Gehoord:** (hoogstens vier, de sterkste eerst), bijvoorbeeld 702 Hz.
4. Onder de knoppen verschijnt de tekst met de toon en de snelheid. Roepletters in het resultaat zijn aan te klikken, net als in het gesprek.

Een aparte decoder doet dit, dus de live decoder loopt ongestoord door. Hij leest het stuk twee keer: de eerste keer alleen om de niveaus te vinden, zodat ook de eerste tekens goed uitkomen.

**Bewaren als WAV.** Klik op **Laatste minuut bewaren (WAV)**. CWLD schrijft de laatste minuut, zoals de decoder hem hoorde (8 kHz, mono, vóór de versterking), naar de opnamemap (standaard Documenten\CWLD). De bestandsnaam bevat datum, tijd en frequentie.

![Terugspoelen](img/nl-listen-again.png)

## 38. Zendteksten bewerken (0.20.1)

Elke chip en elke functietoets is aan te passen, ook de contesttoetsen van de ingebouwde profielen.

1. Klik met rechts op een chip of toets, of klik op het potloodje (✎) achter de chips of de contesttoetsen. Het venster **Teksten van chips en toetsen** opent.
2. Kies bovenaan de groep: **Chips**, **Pile-up F1 tot F4** of **Contesttoetsen F1 tot F8** (per contestprofiel, voor Run en voor S&P; heeft een profiel twee uitwisselingen, dan heeft F2 een tekst voor elk).
3. Kies links het item, pas rechts de **Naam op de knop** en de **Tekst die verzonden wordt** aan.
4. Klik op een variabele onder **Invoegen** ({CALL}, {MYCALL}, {NAME}, {QTH}, {RST}, {NR}, {MYNAME}, {MYQTH} en je eigen contestvariabelen) om ze in de tekst te zetten.
5. Onder **Nu verzonden als** zie je de tekst met de huidige waarden, variabelen zonder waarde zijn gemarkeerd, en de duur bij je huidige snelheid.
6. Klik op **Opslaan**. **Ongedaan** zet je laatste wijziging terug.

CWLD waarschuwt bij een onbekende variabele, tekens die de radio niet kan zenden, een onbekend prosign en een losse accolade. **Standaard herstellen** werkt per item en voor een hele groep. Alleen wijzigingen worden bewaard, zodat latere verbeteringen aan de ingebouwde teksten je toch bereiken. Eigen chips maak je met **+ Eigen chip** en je hernoemt, verschuift (pijltjes omhoog en omlaag) of verwijdert ze. Sluit of wissel je met een niet-opgeslagen wijziging, dan vraagt CWLD eerst.

Chips en toetsen die niet kunnen zenden (LUISTEREN) zijn grijs, maar een rechtsklik opent nog steeds hun tekst.

![Teksten van chips en toetsen](img/nl-texts.png)

## 39. CQ herhalen (0.20.1)

De knop **CQ herhalen (Alt+R)** roept CQ, luistert met een aftelling en roept opnieuw.

- Wat er gezonden wordt: de chip CQ. In contestmodus is dat de F1-tekst in Run (in S&P is er geen CQ).
- Instellingen: onder **CQ herhalen en de paddle** in het tekstenvenster. Pauze 2 tot 60 seconden (standaard 6) en hoogstens 1 tot 30 keer na elkaar (standaard 10).
- Het start **nooit vanzelf**. Het stopt bij elke toets, bij de paddle, als er in een pauze twee of meer tekens te horen zijn (een antwoord), als je iets anders zendt, aan de VFO draait, van mode wisselt, op Stop drukt, CWLD minimaliseert, bij een SWR-alarm en na de laatste keer. De regel onder het zendvak zegt waarom het stopte.
- Zenden kan alleen in LIVE. CQ herhalen zendt alleen zolang jij aan het station zit.

## 40. De paddle gaat voor (0.20.1)

Tik je een tekst of klik je een chip terwijl je met de paddle seint, dan wacht die tekst. CWLD merkt de paddle zo: de radio zendt terwijl CWLD niets zond, of de decoder of de radio (QMX) meldt paddletekst. De tekst staat onder **Wacht op de paddle** en gaat weg zodra de paddle 14 puntlengtes (minstens een seconde) stil is. Met Esc wis je de wachtrij. Je kunt dit uitzetten in het tekstenvenster.

## 41. Split (0.20.2)

Met split luistert de radio op de ene frequentie en zendt ze op een andere, zoals een DX-station in een pile-up vraagt (bv. "UP 1").

**Vanuit een spot.** CWLD leest de opmerking bij de spot. Begrepen worden: `UP`, `UP 1`, `UP1`, `UP 1.5`, `UP 1-2`, `UP 1 TO 2`, `DWN 2`, `DN 2`, `DOWN 2`, `QSX 7027.5`, `QSX 7.0275` en `QSX 27.5`. Zo'n spot krijgt een oranje badge op zijn label (bv. `UP 1`). **Eén klik** op het label zet de ontvangstfrequentie, de zendfrequentie en split. `UP` zonder getal telt als 1 kHz en de balk zegt dat. Een opmerking van een mens blijft 20 minuten bewaard terwijl alleen skimmers het station melden.

**Met de hand.**
1. Klik op **SPLIT** naast de frequentie bovenaan.
2. Typ in de balk `UP 1`, `DWN 2`, `+1.5`, `-2` of een frequentie zoals `7027.5` en druk op Enter. Of klik op een van de knoppen **UP 1**, **UP 2**, **UP 5**, **DWN 1**.
3. Of houd **Shift** ingedrukt en klik op de bandscoop: de radio zendt daar.

Terwijl split aan staat zegt een oranje balk: je luistert op ... en zendt op ... Met knoppen verschuif je de zendfrequentie (0,1 en 0,5 kHz), je typt een andere frequentie, en met **Split uit** schakel je terug. De statusbalk toont `SPLIT TX` met de frequentie. De bandscoop toont een gestreepte oranje lijn met TX (en een gearceerd stuk als het station over een bereik luistert, zoals bij UP 1-2).

**Split op de radio zelf** (de knop SPLIT, de andere VFO) zie je ook in CWLD: CWLD leest ongeveer twee keer per seconde de splitstand en de zendfrequentie. De radio meldt dit niet vanzelf, dus uitzetten gebeurt best vanuit CWLD.

**Veiligheid.** Split instellen zendt niets en mag in elke mode. CWLD weigert een zendfrequentie buiten de amateurband van de ontvangstfrequentie en verandert niets terwijl de radio zendt. Verlaat je de band, of klik je een spot van een station dat geen split werkt, dan zet CWLD split uit en zegt waarom. **Split uit** werkt altijd. Esc blijft de noodstop, ook met de cursor in het splitvak.

**Radiocommando's.** Icom: uit de CI-V Reference Guide van de IC-7300 MK II (oktober 2025): `0F` leest en zet split, `25 01` zet de frequentie van de zend-VFO, `26 01` geeft hem de mode en het filter van de ontvangst-VFO. QMX: `FB` en `SP1` of `SP0`.

**Nog niet op de radio getest**: split op de IC-7300 MK II (tests A68, A69, A70) en op de QMX. Meld het als iets niet klopt.

![Split](img/nl-split.png)

## 42. Logboek met zend- en ontvangstfrequentie (0.20.3)

Het logboek bewaart twee frequenties per QSO: **TX kHz** (waar je zond) en **RX kHz** (waar je luisterde). Meestal zijn ze gelijk. Bij een QSO in split verschillen ze: staat het DX-station op 7025,0 en roep jij op 7026,0, dan is TX 7026,0 en RX 7025,0.

- Het logboekvenster toont beide kolommen standaard. Had je eerder eigen kolommen gekozen, vink dan eenmalig **RX kHz** aan onder **Kolommen**. De kolom die vroeger "kHz" heette, heet nu "TX kHz".
- Het wijzigvenster heeft een veld **RX kHz**. Laat het leeg als je luisterde waar je zond.
- ADIF-export: `FREQ` is de zendfrequentie, zoals de ADIF-standaard zegt. Een QSO in split krijgt ook `FREQ_RX` (en `BAND_RX` als de banden verschillen). Een gewoon QSO wordt precies zo geschreven als vroeger.
- Het N1MM-bericht naar andere programma's bevat de echte `rxfreq` en `txfreq`. De CSV-export volgt de kolommen.
- Bestaande logboekbestanden worden vanzelf bijgewerkt. Oude QSO's blijven zoals ze waren.
- Band en dupecontrole gaan op de zendfrequentie. Split kan alleen binnen één band.

![Logboek](img/nl-logbook.png)

## 43. Land, richting en afstand (0.20.4)

Bij een roepletter toont CWLD het land, de richting en de afstand, in het QSO-paneel, in de contest-invoerlijn en als tooltip op de roepletters in het gesprek. Bijvoorbeeld `Germany  EU  75°  ~407 km`.

- Het land komt uit een prefixtabel in CWLD (de langste prefix wint: `KH6` is Hawaï en `K` de Verenigde Staten). Portabele roepletters worden begrepen: `ON3VZ/P` is België, `EA8/ON3VZ` en `ON3VZ/EA8` zijn de Canarische Eilanden, `/MM` en `/AM` horen bij geen land.
- Richting (korte weg) en afstand gaan vanaf je eigen locator: die van de activatie, anders van het stationsprofiel, anders van de instellingen. Zonder locator zie je alleen het land en een hint. Vul je locator in bij Instellingen, Station.
- Gaf QRZ.com een locator voor het station, dan gaan richting en afstand daarheen. Anders gaan ze naar het midden van het land en staat er een `~` voor de afstand, want dat is maar ruw. De lange weg staat in de tooltip.
- Een station in je eigen land toont "je eigen land" en geen afstand, tenzij QRZ.com een locator gaf.
- Alles staat alleen op het scherm. Er wordt niets van naar het logboek, een ander programma of een ander station gestuurd.
- De tabel is met de hand gemaakt (ongeveer 300 entiteiten) en kan gaten hebben, vooral bij speciale prefixen. Het is een gewoon gegevensbestand (`ui/src/lib/dxcc-data.ts`); `node tools/update_prefixes.mjs cty.dat` bouwt hem opnieuw uit een Country Files-lijst (kijk eerst de licentie na). De versie van de tabel staat in de tooltip.

## 44. Sneltoetsen zoals N1MM Logger+ (0.20.4)

Deze toetsen starten nooit zolang een venster zoals Instellingen of het tekstenvenster openstaat. Niets zendt zonder toetsaanslag, en CWLD zendt nog steeds alleen in LIVE.

| Toets | Wat hij doet |
|---|---|
| **Insert** | Zendt zijn roepletters en jouw uitwisseling in één keer (contestmodus). Begint de uitwisselingstekst al met `{CALL}`, zoals bij de ingebouwde, dan gaat alleen de uitwisseling uit, zodat de roepletters niet dubbel gaan. |
| **.** (punt) | Zendt TU (F3) en logt het QSO, maar alleen als de uitwisseling compleet is, zoals Enter. Buiten contestmodus logt hij alleen. In een tekstvak (zendvak, naam, splitvak) blijft de punt een teken. Werkt met Shift (AZERTY) en met de punt van het numerieke blok. |
| **PgUp / PgDn** | Snelheid 2 WPM sneller of trager, tussen 5 en 60 WPM. Enkel de snelheid van de keyer verandert. |
| **Ctrl+pijl omhoog / omlaag** | Stemt af op de volgende spot boven of onder je op de huidige band en zet zijn roepletters in de invoerlijn. Gewerkte stations en spots ouder dan de clusterlimiet worden overgeslagen. Een spot met "UP 1" zet ook split, zoals een klik. |
| **Alt+Q** | Terug naar de frequentie waar je het laatst CQ riep, met split uit. CWLD onthoudt ze als je CQ zendt (chip, getypt, CQ herhalen, F1 in Run). Een knop "CQ 7025.0 (Alt+Q)" staat naast Stop zodra je op een andere frequentie zit. |
| **Alt+F7** | Split uit als hij aan staat, anders opent hij de splitbalk (zoals de knop SPLIT). |

Insert, de punt, Alt+Q, Ctrl+pijl en Alt+F7 doen niets als je de toets ingedrukt houdt, zodat een QSO nooit per ongeluk dubbel gezonden of gelogd wordt.

**Botsingen.** Er is geen botsing met de bestaande toetsen: F1 tot F8, F6, F7, Ctrl+F6, Ctrl+F7, Ctrl+R, Ctrl+W, Ctrl+Q, Alt+R, S en Esc. De gewone F-toetsen negeren nu Alt, dus Alt+F7 zet AFC niet om. Shift+klik op de bandscoop is een muisactie en botst niet.

**Nog niet op de radio getest**: tests A72, A73 en A74.

## 45. Melding in contestmodus over de decoder (0.20.4)

In sommige contests telt een decoder als hulp (een aparte klasse Assisted) of is hij helemaal niet toegestaan. Daarom toont CWLD elke keer dat je contestmodus aanzet een herinnering, met een knop om ze te sluiten. Ze staat alleen op het scherm: er gaat niets in het logboek en niets naar een ander station. Lees het reglement van de contest en kies zelf de juiste klasse.

## 46. Kleine verbeteringen (0.18.2 tot 0.19.3)

- **Exporteren (0.18.5).** In het exportvenster loopt de controle nu vanzelf als je op **Exporteren** klikt; vroeger bleef de knop grijs tot je op Controleren drukte. Lukt het niet, dan zegt het venster waarom, in gewone woorden. Een export zonder QSO wordt geweigerd (vroeger kwam er een leeg bestand). Na de export staan het pad en de knop **Map openen** in het venster.
- **Paddle oefenen (0.18.3).** Lange pauzes tussen tekens deden juiste tekens als fout tellen; dat is hersteld. Tekens worden nu zonder de spaties vergeleken. De spatiëring heeft haar eigen tabel en tips.
- **Neuraal netwerk (0.18.3).** Een klein netwerk is uitgeprobeerd als extra bron voor de decoder. Op de testbank gaf het geen winst en het staat uit. De code blijft in de repository voor later, met echte opnames.
- **Notchmelding (0.18.4).** De toonzoeker volgt de rand van een filter of notch niet meer. Zie ook hoofdstuk 31.
- **Het logboekvenster en CSV** staan in hoofdstuk 32 en 35, bewerken van een QSO in hoofdstuk 34.
