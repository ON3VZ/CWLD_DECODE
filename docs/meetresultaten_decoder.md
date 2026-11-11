# CWLD decoder: meetresultaten (fase 3, versie 2)

Let op: dit zijn metingen op **synthetische CW**. Ze zeggen nog niets over echte banden en nog niets over MRP40 of CwGet. Die vergelijking volgt met echte opnames van Kristofs radio (testplan I10 en Q10).

## Wat gemeten is

Tekenfoutpercentage (CER): bewerkingsafstand tussen gezonden en gedecodeerde tekst, gedeeld door de lengte van de gezonden tekst. Spaties tellen mee. Boven 100 procent betekent: meer fouten dan tekens, de decoder verzint tekst uit ruis. SNR is het vermogen van de toon bij neergedrukte sleutel tegenover de ruis in 500 Hz. Per cel 12 berichten uit een vaste set van 10 QSO-zinnen, sleutelfout 5 procent.

Elke cel toont **voor / na**. "Voor" is dezelfde decoder met de nieuwe onderdelen uitgeschakeld (`--plain`), "na" is de standaardinstelling. Herhalen: `cargo run --release -p cwld-decoder --bin cwld_bench` en hetzelfde met `-- --plain`.

| SNR dB | 12 WPM | 20 WPM | 28 WPM | 36 WPM |
|---|---|---|---|---|
| +30 | 0,0 / 0,0 | 0,0 / 0,0 | 0,0 / 0,9 | 0,0 / 0,0 |
| +15 | 0,0 / 0,0 | 0,0 / 0,0 | 0,0 / 0,0 | 0,0 / 0,0 |
| +10 | 0,0 / 0,0 | 0,0 / 0,0 | 0,0 / 0,0 | 0,0 / 0,0 |
| +6 | 0,0 / 0,0 | 0,0 / 0,0 | 0,0 / 0,0 | 0,0 / 0,0 |
| +3 | 0,0 / 0,0 | 0,0 / 0,0 | 0,6 / 0,9 | 1,1 / 0,7 |
| +0 | 1,5 / 1,5 | 4,6 / 4,0 | 11,9 / 11,6 | 20,1 / 18,0 |
| -3 | 22,4 / 18,0 | 52,5 / 49,8 | 66,1 / 60,4 | 72,2 / 66,2 |
| -6 | 152,2 / 134,9 | 129,4 / 108,9 | 97,9 / 96,4 | 89,4 / 84,5 |

| Moeilijk signaal (20 WPM, +10 dB tenzij vermeld) | Voor | Na |
|---|---|---|
| hand keying, 15% jitter | 2,4 | 3,0 |
| sloppy spacing x1.5 | 0,7 | 0,3 |
| short dashes (2.5) | 0,0 | 0,0 |
| long dashes (3.6) | 0,0 | 0,0 |
| deep QSB (0.8) | 43,6 | 35,1 |
| tone drift 20 Hz | 0,0 | 0,0 |
| mistuned by 25 Hz | 0,0 | 0,0 |
| second station 60 Hz away, equal level | 0,0 | 1,0 |
| second station 120 Hz away, +10 dB | 0,0 | 0,0 |
| impulse noise, 15 clicks/s at +46 dB | 87,5 | 0,3 |
| impulse noise, 40 clicks/s at +40 dB, SNR +6 | 196,6 | 2,4 |
| carrier 50 Hz away, +15 dB (present 3 s before) | 100,0 | 2,7 |
| carrier 120 Hz away, +25 dB (present 3 s before) | 100,0 | 1,4 |
| carrier 50 Hz away, +15 dB, starts with the signal | 99,3 | 11,5 |
| deep QSB at +3 dB | 59,8 | 56,1 |
| slow 8 WPM at 0 dB | 0,7 | 1,4 |
| fast 45 WPM at 15 dB | 0,0 | 0,0 |

## Wat er nieuw is

1. **Noise blanker** (spoor 2): korte, sterke klikken (ontsteking, schakelende voedingen) worden stilgemaakt. Alles wat langer duurt dan 6 ms is signaal en blijft staan, want een punt duurt bij 50 WPM al 24 ms. Bij zware klikstoring van bijna 200 procent fouten naar ongeveer 2 procent.
2. **Automatische notch** (spoor 2): een vaste draaggolf binnen 400 Hz van de toon wordt gevonden en weggefilterd. Een draaggolf verandert bijna niet tussen blokken van een kwart seconde, een gesleuteld station wel, dus een tweede CW-station wordt niet weggefilterd. Van volledig onleesbaar naar 1 tot 3 procent fouten als de draaggolf er al was. Begint de draaggolf tegelijk met het signaal, dan gaat de eerste anderhalve seconde verloren (ongeveer 11 procent).
3. **Fading volgen** (spoor 1): het niveau van de neergedrukte sleutel wordt snel gevolgd, zodat een dal in de QSB niet als "sleutel los" telt. Diepe QSB van 44 naar 35 procent.
4. **Woordmodel** (spoor 1): Q-codes, afkortingen, roepletterpatronen en getallen kiezen tussen twijfelachtige lezingen van hetzelfde woord. Een zeker teken wordt nooit veranderd. Losse E's of T's die nauwelijks boven de ruis uitkwamen worden weggelaten in plaats van getoond. De woordenlijst is algemeen CW-taalgebruik. Ze bevat geen namen of plaatsen uit de testberichten, maar wel woorden als RAIN, COLD en DIPOLE die in de testberichten staan: de winst daar is dus iets optimistisch.
5. **Snelheid**: lange pauzes worden nu zonder extra rekenwerk behandeld. De decoder verwerkt 13 seconden audio in minder dan een seconde, ook met storing.

## Wat geprobeerd en niet gehouden is

Het exacte ruismodel voor een toon in ruis (Rice tegenover Rayleigh) is gebouwd en getest. Het is beter gekalibreerd, maar gaf in de meting meer fouten dan het eenvoudigere model. Het blijft als optie in de code (`rice`), standaard uit.

## Wat het betekent

* Vanaf +3 dB blijft de decoder op dit testmateriaal vrijwel foutloos, nu ook met klikstoring en met een draaggolf ernaast.
* Rond 0 dB: 2 tot 18 procent fouten afhankelijk van de snelheid.
* Bij -3 dB is er een kleine verbetering (bijvoorbeeld 22 naar 18 procent op 12 WPM), maar de decoder blijft daar zwak. Onder -3 dB verzint hij nog steeds tekst.
* Diepe fading blijft moeilijk. In de dalen is het signaal echt te zwak; een deel daarvan is een natuurkundige grens, geen fout van het algoritme.

## Wat nog moet

1. Het opsplitsen van strepen bij zwakke signalen (een korte uitval midden in een streep geeft twee tekens). Dit is nu de grootste bron van fouten onder 0 dB.
2. Echte opnames als testmateriaal en een eerlijke vergelijking met CwGet via WAV-bestanden.
3. Audio-ingang (cpal), koppeling met de waterfall, het gesprek en de zekerheidsmeter.
4. Het woordmodel aanvullen met eigen roepletters, naam en QTH en met roepletters uit het logboek.


## Echte opnames, IC-7300 MK II (4 november 2026)

Drie opnames van Kristof via de REC-knop, 48 kHz, teruggebracht naar 8 kHz. De juiste tekst is niet bekend, dus dit is een vergelijking op herkenbare woorden en op hoeveel tekst er uit ruis komt.

| Opname | Wat erin zit | Vroeger (700 Hz vast, drempel 2) | Nu (700 Hz start, zelf zoeken, drempel 3,5) |
|---|---|---|---|
| 7047,2 kHz, 60 s | Bijna lege band, alleen ruis | 122 tekens onzin | 11 tekens (`M EETE I M T T TT`) |
| 7035,4 kHz, 26 s | OK5O roept CQ CWT op 799 Hz, G5RMP antwoordt op 640 Hz | `E G5 M A O I M O GECQ CWT OK5O OK5O G5RMP` | `E G5 GH M CQ CWT OK5O OK5O MTMT MMTM MTMT TMM M TM K5O OK5O G5RMP` |
| 7012,0 kHz, 60 s | Frans QSO, F8UFT en partner rond 610 Hz | Bijna volledig leesbaar (valt binnen de AFC) | Gelijk |

Synthetisch signaal op 615 Hz terwijl de decoder op 700 Hz begint (20 WPM, 6 berichten):

| SNR | Zonder zoeken | Met zoeken |
|---|---|---|
| +3 dB | 33,6 % | 7,5 % |
| 0 dB | 69,2 % | 54,1 % |
| -3 dB | 82,2 % | 93,8 % |

Synthetisch op de juiste toon, drempel 2 tegen 3,5 (8 berichten, 20 WPM): +6 en +3 dB 0 % in beide, 0 dB 3,9 % in beide, -3 dB 42,4 % tegen 55,2 %. De hogere drempel kost dus iets bij de zwakste signalen; wie die wil lezen zet de drempel lager.

Wat de opnames leerden:

- De ruis van de radio is gekleurd door het CW-filter. De oude aanwezigheidstest (toonband tegen twee buurbanden) zei daardoor bijna altijd "signaal", en de scheiding van ruis alleen is op de radio ongeveer 3,3. Daarom de drempel 3,5.
- De signalen zaten op 410, 615, 640 en 799 Hz, niet op 700. Zonder zoeken bleef de decoder op de verkeerde plaats.
- Het stuk `MTMT MMTM` in de OK5O-opname is nog niet verklaard (zelfde stuk ook bij vaste 796 Hz). Daarvoor is een opname met bekende tekst nodig.
- Zoeken vindt signalen vanaf ongeveer +3 dB. Zwakkere signalen moet je aanklikken.

## Opname 7008.0 kHz (DL6JF, 22 tot 26 WPM): snelheid liep weg in een pauze

- Zuiver signaal op 580 Hz, punt 54 ms (22 WPM), later 46 ms (26 WPM), streep 3 eenheden, korte lettergaten (ongeveer 2,2 eenheden).
- Fout: na "K" volgde 3,5 s stilte. In die stilte schoof de snelheidsschatting bij elke zoekronde iets omhoog (35, 46, 55, 71, 84, 110 WPM). De volgende uitzending kwam eruit als losse letters (`MTMT MMTM MTMT MMTT` voor `CQ CQ`). Dit is ook de verklaring voor het patroon in de OK5O-opname.
- Oplossing: de snelheid volgt alleen nog echte tekens (minstens 2 gesloten tekens in het pad). Resultaat op dezelfde opname: `K CQCQDEDL6?DL6JF PS`, snelheid blijft 26 tot 27 WPM tijdens de pauze.
- Waterfall: de ruisreferentie was het 25e percentiel over de hele strook. Bij een sterk signaal met brede zijbanden of een smal radiofilter werd het hele midden wit. Nu een lopende mediaan per frequentiebin (mediaan van de buren, 80 Hz aan elke kant). Op deze opname: één dunne lijn op 580 Hz.

## Opname 0,0 kHz (19:18, alleen ruis, pitch 956 Hz): tekst uit ruis

- Gebruiker: de decoder gaf een stroom letters (`TTTT MTMT EE`) met 94 procent zekerheid, terwijl er niets te horen was. Opname 33,5 s, 48 kHz.
- Spectrum van de opname: 37 tot 40 dB (relatief) tussen 200 en 800 Hz, 21 dB bij 800 tot 1000 Hz, onder -7 dB boven 1000 Hz. De pitch staat op de steile flank van het radiofilter. De scheiding van niveau (`separation`) schommelde daar tussen 3,2 en 3,7 voor pure ruis, rond de drempel 3,5. Een niveautest alleen kan ruis hier niet van een zwak signaal scheiden.
- Meting van de vorm van de omhullende (5 s venster, drempel halverwege tussen het 25e en 97e percentiel, bandbreedte van het filter als schaal). Aandeel merken korter dan de reactietijd van het filter:

| Bron | Aandeel korte merken |
|---|---|
| Deze ruisopname (956 Hz en 700 Hz) | 0,97 tot 1,00 |
| Ruisopname 7047,2 kHz (700 en 410 Hz) | 0,98 tot 1,00 |
| Witte ruis, synthetisch | 0,96 tot 1,00 |
| Echt signaal, F8UFT 7012 kHz | 0,00 tot 0,50 (meestal 0,05) |
| Synthetisch +10 / +3 dB (12, 20, 30 WPM) | 0,00 tot 0,10 / 0,07 tot 0,56 |
| Synthetisch 0 dB | 0,21 tot 0,81 |
| Synthetisch -3 dB | 0,48 tot 0,97 |

- Maatregel: de poort sluit na 3 opeenvolgende metingen (0,25 s) met minstens 0,95 en 8 merken, en opent weer bij 0,90 of lager. Er wordt dan niets gedecodeerd en de zekerheidsmeter toont geen waarde. Bij +3 dB en beter is er geen overlap met ruis. Bij -3 dB en zwakker overlapt het wel (tot 0,97), dus daar kan de poort dicht blijven. Die signalen werden al niet betrouwbaar gelezen.
- Resultaat op de ruisopname: 0 tekens (voorheen een stroom onzin op 956 en op 700 Hz). Echte opnames: F8UFT en OK5O nog leesbaar.
- Tabel (CER, 4 berichten per cel, `cwld_bench --quick`, oud tegenover nieuw): +30 tot +3 dB onveranderd. 0 dB: 12 WPM 2,4 naar 0,0, andere snelheden gelijk. -3 dB: 17,6/51,0/55,0/72,2 naar 18,8/52,0/50,8/71,1. -6 dB: 96,5/117,3/93,3/84,4 naar 52,9/91,8/92,5/100,0 (minder verzonnen tekst). Moeilijke gevallen: tweede station 60 Hz verder 0,0 naar 1,9, traag 8 WPM bij 0 dB 1,9 naar 3,8, carrier 120 Hz 1,9 naar 4,7. Dat zijn verschillen van een enkel teken bij 4 berichten per cel, dus binnen de meetonzekerheid, maar niet bewezen gelijk. Een run met 12 berichten per cel volgt bij de volgende meting.
- Nog niet gemeten: bandruis met sterke pieken (atmosferische klikken), kamerruis in de microfoon, en een zwak signaal dat net begint na lange ruis (de start wordt 3 s bewaard, daarna weggegooid).

## Spraak en SSB (0.12.0): geen tekst meer uit een telefoniesignaal

- Gebruiker: staat de radio op SSB of op een verkeerde frequentie, dan probeert de decoder toch te lezen en komt er onzin uit.
- Testsignaal: synthetische SSB-spraak (`cwld-dsp`, `generate_voice`): stemhebbende bron met een toonhoogte die tussen 90 en 220 Hz glijdt, harmonischen gevormd door twee bewegende formanten, lettergrepen van 80 tot 300 ms, pauzes, wat ruis voor stemloze klanken, doorlaat 300 tot 2700 Hz. Geen echte spraak; een opname van een echt SSB-station is nog nodig.
- Kenmerk: bij CW wordt alleen de eigen toonhoogte luider. Bij spraak gaan de toonhoogte van de decoder en de rest van de doorlaatband (300 tot 2700 Hz, zonder 200 Hz aan elke kant) samen op en neer. Om de 32 ms worden beide niveaus gemeten (Blackman-venster, klikken boven 10 keer de mediaan uitgeblankt), en over 4 s de correlatie berekend, samen met het niveauverschil op de luidste momenten.
- Meting van het kenmerk (correlatie, 4 opnames per geval):

| Bron | Mediaan | Hoogste |
|---|---|---|
| Spraak +30 dB | 0,84 | |
| Spraak +10 dB | 0,54 tot 0,57 | |
| Spraak +3 dB | 0,43 tot 0,45 | |
| Spraak 0 dB | 0,31 tot 0,33 | |
| CW +20, +3, -3 dB, QSB, tweede station, draaggolf, klikken, 12 en 35 WPM | -0,02 tot 0,03 | 0,41 |
| Klikken 15 per s op +46 dB en 40 per s op +40 dB | 0,11 tot 0,13 | 0,40 |
| Zuivere toon zonder ruis (zijtoon) | 0,99 | (maar het niveauverschil is -86 dB, dus geen spraak) |

- Beslissing: spraak als de correlatie minstens 0,45 is en de rest van de band op de luidste momenten hoogstens 22 dB onder de toonhoogte zit. Na spraak start het decoderen pas weer na 6 heldere metingen (ongeveer 1,5 s). Geen enkel CW-geval werd ooit als spraak gezien.
- Resultaat, tekens per minuut uit spraak (5 opnames van 60 s, gemiddeld, juist afgestemd en 150 Hz ernaast):

| SNR | Voor | Na |
|---|---|---|
| +30 dB | 43 / 36 | 0 / 0 |
| +20 dB | 54 / 50 | 0 / 0,4 |
| +10 dB | 46 / 44 | 4,0 / 5,2 |
| +3 dB | 43 / 42 | 14 / 9 |

- CW-testbank (`cwld_bench`, 12 berichten per cel) met en zonder deze poort: **identiek tot op het cijfer**, ook alle moeilijke gevallen. Een eerste versie zonder klikblanker sneed de twee gevallen met sterke klikken af (100 procent fouten); de blanker loste dat op.
- Daarnaast een harde regel: staat de radio op USB, LSB, AM, FM of RTTY, dan toont CWLD geen tekst en meldt dat boven het gesprek. Bij OEFENEN niet, want dan komt het geluid van de oefenband.
- Te zwakke CW gaf al geen tekst (bij -10 dB en lager 0 tekens, bij -6 dB gemiddeld 1,4 tekens per bericht met de standaarddrempel).
- Nog te doen: een opname van een echt SSB-station en van CW met een sterk SSB-station ernaast. Is de spraak zelf op de toonhoogte van de decoder, dan wacht de decoder ook voor een CW-signaal in die spraak; dat is de bewuste keuze.

## Paddle oefenen: zijtoon en sleuteltijden (0.13.2)

Aanleiding: ON3VZ zag de zijtoon van zijn paddle in de audio van CWLD, maar de paddle-kaart herkende niets. Nagebootst met de testbank (`paddle_harmonics_experiment` en `paddle_keying` in crates/cwld-decoder/src/pipeline.rs), zijtoon 600 Hz, decoder start op 700 Hz, 20 WPM.

| Zijtoon | Tekst voor (0.13.1) | Tekst na (0.13.2) |
|---|---|---|
| Zuivere sinus | NARIS KMRSU PARIS | NARIS KMRSU PARIS (score leest nu PARIS uit de sleuteltijden) |
| 2de boventoon op -20 dB | NARIS KMRSU PARIS | idem |
| 3de en 5de boventoon (blokachtig) | geen tekst: spraakcontrole hield de decoder vast | NARIS KMRSU PARIS |
| Hard geknipt op 30 procent | geen tekst | NARIS KMRSU PARIS |

Sleuteltijden (aantal key-downs, PARIS KMRSU heeft er 28):

| Situatie | Voor | Na |
|---|---|---|
| Zuivere zijtoon, 0,5 s stilte vooraf | 34 (heraudio na toonwissel dubbel geteld) | 28 |
| Zuivere zijtoon, 10 s stilte vooraf | 34 | 28 |
| Bandruis 40 dB en 20 dB, 10 s ruis vooraf | 179 tot 210 | 28 |
| Alleen ruis, 20 dB, 8 s | veel valse key-downs | 0 |
| Bandruis 10 dB | niet gemeten | eerste element van P gemist |

Elke key-down valt binnen 15 ms van 60 ms (punt) of 180 ms (streep). Met de squelch dicht (de standaard in OEFENEN) is er geen ruis, dus dit is het gewone geval.

## Slordige vuisten en echte opnames van 10 november (0.15.0)

Aanleiding: ON3VZ meldde dat het decoderen beter kan bij zwakke en vooral slordige morse, en stuurde zeven opnames (7 en 3,5 MHz, 15 tot 43 s). In alle zeven steekt de toon 25 tot 32 dB boven de ruis uit. De fouten kwamen dus van de spatiëring en het ritme, niet van ruis. De juiste tekst van de opnames is niet bekend: wat hieronder staat is een vergelijking op herkenbare woorden, geen foutpercentage.

### Nieuwe gevallen in de testbank

`cwld_bench` heeft een derde tabel "Sloppy fists" (alleen die tabel: `cwld_bench --fists`). De generator kan nu de pauze binnen een teken, tussen tekens en tussen woorden apart instellen, de pauzes apart laten spreiden en de snelheid laten schommelen. 20 WPM, +15 dB, 12 berichten per geval, tenzij vermeld. CER in procent, voor (0.14.0) en na (0.15.0):

| Geval | Voor | Na |
|---|---|---|
| Letters ver uit elkaar (teken 6, woord 14 eenheden), zoals een naam spellen | 43,4 | 0,0 |
| Beginner met lange pauzes, Farnsworth (teken 9, woord 21) | 52,8 | 2,4 |
| Woorden aan elkaar (woordpauze 4,5) | 21,5 | 1,8 |
| Tekens aan elkaar (tekenpauze 2,2) | 0,6 | 0,0 |
| Zware weging (pauze binnen een teken 1,8, teken 5, woord 10) | 56,9 | 1,2 |
| Lichte weging (pauze binnen een teken 0,6) | 0,0 | 0,0 |
| Ongelijke pauzes, 30 procent spreiding | 22,4 | 15,9 |
| Ongelijke pauzes 30 procent en elementen 15 procent (seinsleutel) | 28,0 | 21,8 |
| Bug: lange strepen (4,2) en strakke punten, teken 4 | 2,7 | 0,3 |
| Snelheid schommelt 15 procent | 0,3 | 0,0 |
| Snelheid schommelt 25 procent, ongelijke pauzes | 5,6 | 2,7 |
| Letters ver uit elkaar op 30 WPM (teken 6, woord 13) | 44,8 | 0,0 |
| Zware weging op 35 WPM, +6 dB | 70,5 | 0,6 |
| Ongelijke pauzes 30 procent bij +3 dB | 24,2 | 15,3 |

Zeer ongelijke pauzes blijven moeilijk: bij 30 procent spreiding overlappen de pauze binnen een teken en de pauze tussen tekens echt, en dan helpt alleen woordkennis.

### De gewone testbank (zelfde signalen als hierboven, 12 berichten per cel)

Niets werd duidelijk slechter. Verschillen voor en na: 0 dB 20 WPM 5,0 naar 0,7; 0 dB 36 WPM 18,0 naar 15,8; -3 dB 12 WPM 19,1 naar 14,3; -3 dB overige snelheden 1 tot 3 punten beter; -6 dB 12 WPM 70,2 naar 62,9; klikstoring 40 per s 2,4 naar 0,3; 15 procent jitter 3,0 naar 2,4; diepe QSB 33,8 naar 34,5 en bij +3 dB 44,6 naar 43,6 (binnen de meetonzekerheid); draaggolf 120 Hz verder 4,7 naar 6,1 (één tot twee tekens, niet verklaard). Alle andere cellen zijn gelijk, de cellen vanaf +6 dB blijven 0,0.

### De zeven opnames, decoder start op 700 Hz met "Sterkste" aan

| Opname | Voor (0.14.0) | Na (0.15.0) |
|---|---|---|
| 7009,0 kHz | `YNAN E LARRY LARRY LARR` | `YNAN E WNAG NAG LARRY LARRY LARAE` |
| 7011,7 kHz | `IMY RM3Y CQ NNQ CQDE EN M3Y ENME TT ET N K` | `CQ CQ DE RM IM Y R M3Y CQ CQ CQ DE EN M3Y ENM MY U N K` |
| 7008,9 kHz | `E T E OR N Y T H O I N Y I OR N 0 S M IEE T H OR N Y BK BKOKDRTS@NYFB RB ==SORRYLOTSOF?M` | `K TEORNY THOINY IOENN2 IEE M IEE THORNY BK BK O?RTS@-FBRB ==S:?LOTSOF?M` |
| 7005,0 kHz | `IUKT E SUKY GM MOT W 559 55N QSB NICE TO MEET YE EU MY FRIEND MO T T` | `IUKT UKY JI UKY GM MOT M 559 55N QSB NICE TO MEET YE EU MY FRIEND MO TT` |
| 3552,1 kHz | `Q CQ CQ DE IK1QHB ITT1QHB IK1QHB PSE K E KE` | `Q CQ CQ DE IK1QHB ITT1QHB IK1QHB PSE K ME` |
| 3535,1 kHz | `OPIS A L D O A L D O ALD OESQTH IS P ORE C P M` | `OP IS ALDO ALDO ALDOESQTHIS POREC PM` |
| 3535,0 kHz | `LLW I DE9A2NB K GBDE M0PKD GE.N` | `LL CPI?+ DE 9A2NB K RBDE M0PKD GE.N` |

Wat beter is: gespelde namen blijven één woord (ALDO, THORNY), `CQ DE` wordt gesplitst, het begin van een uitzending gaat niet meer verloren als de decoder eerst de toon moet zoeken (7011,7 kHz: `CQ CQ DE RM...` staat er nu). Wat niet beter of slechter is: in 7009,0 kHz staat het begin nu dubbel (eerst gelezen naast de toon, daarna opnieuw op de toon); in 7008,9 kHz is `SORRY` nu `S:?` (de snelle staart van een zender die eerst traag spelde); `ITT1QHB` (een punt verdween in een fade, K werd TT) blijft; `ALDOESQTHIS` blijft aan elkaar omdat de zender daar zelf geen woordpauze liet.

### Wat er veranderd is

1. **Woordgrenzen** (`spacing.rs`, nieuw). De decoder geeft per teken de stilte ervoor door, en een aparte stap beslist wat een woordpauze is. De drempel volgt de zender (het midden tussen zijn gewone tekenpauze en woordpauze), vijf tekens worden vastgehouden zodat een pauze ook beoordeeld wordt met wat erna komt, en woordkennis beslist bij twijfel: `CQ DE` is twee woorden, vier losse letters zijn bijna nooit vier woorden. Duidelijke timing wint altijd.
2. **Stijl van de zender** (`decoder.rs`). De pauze binnen een teken (weging) en de pauze tussen tekens worden per zender geleerd, uit zijn eerste acht elementen meteen, en daarmee worden die eerste tekens opnieuw gelezen voor er iets getoond wordt. Daarna volgt de decoder de stijl, eerst snel en dan traag.
3. **Te snelle snelheid herkennen.** Een snelheid waarbij bijna elk element een los teken is (alleen E en T) is de verkeerde snelheid: zo'n lezing krijgt strafpunten. Dit loste het geval op waarbij zware weging als een rij E's en T's gelezen werd.
4. **Zelfde punten en strepen, andere grenzen.** Een teken dat door een aarzeling in tweeën brak (5 als S I, R als E N) of twee tekens die samensmolten (T N als G) worden hersteld als dezelfde rij punten en strepen een bekend CW-woord of een pas gehoorde roepletterreeks vormt, en alleen als de pauze op die plaats twijfelachtig was. HANS wordt nooit SIGS, al zijn de elementen gelijk.
5. **Toon zoeken.** Na een toonwissel wordt de audio twee keer afgespeeld: de eerste keer alleen om niveaus en toon te vinden, de tweede keer om te lezen. Een kleine stap terwijl er al tekst uitkomt wordt overgeslagen (de AFC doet dat), anders stond het einde er dubbel. En de eigen toonhoogte telt niet als bezet als het alleen de flank van een veel sterkere toon ernaast is: op een stille band (squelch dicht) bleef de decoder anders 100 Hz naast de zijtoon staan.

Signaal naast de decodertoon (`seekbench`, 10 berichten per cel, CER bij +15, +6 en +3 dB), met één keer afspelen zoals vroeger tegenover de nieuwe aanpak (beide met de rest van 0.15.0, dus dit meet alleen punt 5): 615 Hz op 20 WPM 5,8 2,7 5,8 naar 5,8 2,2 10,6; 520 Hz op 28 WPM 1,8 6,2 11,9 naar 0,0 0,0 6,6; 850 Hz op 16 WPM 1,3 3,5 5,8 naar 0,0 0,0 1,8. De cel 615 Hz bij +3 dB is slechter geworden en niet verklaard.

### Paddle oefenen: zijtoon los van de decodertoon

Fout (ON3VZ, 0.14.0): de zijtoon was te zien en te horen, maar de oefening registreerde niets. Nagebootst: de sleuteldetector las de omhullende op de toon van de decoder. Stond die elders (een eerder aangeklikt signaal, "Sterkste" uit), dan kwam er niets of onzin: decoder op 450 Hz, zijtoon op 600 Hz gaf 10 key-downs in plaats van 21. De detector heeft nu een eigen rij van tien filters van 300 tot 1200 Hz (10 ms venster) en neemt de sterkste. Test `key_times_do_not_depend_on_the_decoder_pitch`: decoder op 450, 500, 600, 700 en 1000 Hz, zijtoon op 400, 600 en 800 Hz, met en zonder ruis: telkens exact 21 key-downs binnen 15 ms. Ruis alleen geeft nog steeds 0. Daarnaast leest de decoder zelf de zijtoon nu ook goed: `PARIS KMRSU PARIS` in plaats van `NARIS KMRSU PARIS`.

### Nog niet gedaan

- De juiste tekst bij de opnames (van ON3VZ), zodat er een foutpercentage op echte signalen komt.
- Een punt of streep die in een fade verdwijnt (ITT1QHB).
- Een zender die binnen één uitzending van spatiëring wisselt (traag spellen, daarna snel).
- De vertraging: definitieve tekst komt nu ongeveer vijf tekens later, of na een stilte van ruim een woordpauze. De voorlopige tekst toont de tekens meteen.

## Verloren punten en strepen, gewoon Engels (0.15.1)

Aanleiding: `ITT1QHB` in de opname van 3552,1 kHz (de punt van de K verdween in een fade), en een nieuwe opname van ON3VZ van 11 november (7026,8 kHz, 65 s, toon 414 Hz, 25 dB boven de ruis): een QSO in gewoon Engels. Met 0.15.0 las de app daar onder meer `LAW TOP`, `G T NEW LAPTOP`, `WENTINTO NORWIFH TO GETIT CHECKED OUT BUT ONLYFIX WUD EE`.

### Wat er veranderd is

1. **Eén verloren element terugzetten.** Als wat gelezen is precies één punt of streep minder heeft dan een roepletterreeks die net gehoord is, of dan een gewoon CW-woord, en de stilte op die plaats is zo lang als het verloren element met de pauzes eromheen, dan wordt dat woord gekozen. Voor een roepletterreeks kost dat 2,0 nat, voor een woord 1,5, plus de afwijking van de verwachte lengte. Zonder passend gat verandert er niets: ON3VI wordt geen ON3VS en DL1AN geen DL1AD (tests `a_callsign_that_only_looks_like_one_heard_before_is_left_alone`). Het eerste en laatste element van een woord worden nooit teruggezet, want daar zegt de stilte niets.
2. **Eigen roepletters** (en wat de app verder als woord doorgeeft) gelden vanaf het begin als "gehoord".
3. **Gewoon Engels in de woordenlijst**: ongeveer 330 veel gebruikte woorden en gangbare QSO-onderwerpen (GET, IT, WENT, INTO, ONLY, FIX, LAPTOP, SCREEN, YESTERDAY). Ze dienen om woordgrenzen te kiezen waar de zender zelf geen duidelijke woordpauze liet.
4. **Minder snel splitsen.** Een kort onbekend stukje (2 tot 4 tekens) kost nu 0,8, zodat BERLIN niet BE RLIN wordt. OP, DE en dergelijke tellen direct na een duidelijke tekenpauze als gewoon woord en niet als los woord (LAPTOP bleef anders LAPT OP). GM, GA, GE, GN, KN, SK, FB en HW zijn geen "altijd losse" woorden meer.
5. **Plaatselijke drempel strenger.** Een kleine stap in de pauzes (1,4 tot 1,8 keer) telt alleen nog boven een strakke groep korte pauzes. Bij een zender wiens pauzes van 2 tot 6 eenheden lopen werd de drempel anders te hoog (ONLYFIX).

### Metingen

Nieuwe gevallen in `cwld_bench --fists` (20 WPM, 12 berichten): de generator laat een deel van de elementen helemaal wegvallen (`lost_elements`).

| Geval | 0.15.0 | 0.15.1 |
|---|---|---|
| 3 op 100 punten en strepen verloren, +15 dB | 13,9 | 11,2 |
| 6 op 100 verloren, +6 dB | 25,7 | 20,1 |

De andere vuisten, 0.15.0 naar 0.15.1: zware weging 1,2 naar 0,0; op 35 WPM 0,6 naar 0,0; bug 0,3 naar 0,0; seinsleutel (30 en 15 procent) 21,8 naar 19,8; snelheid 25 procent 2,7 naar 2,4; woorden aan elkaar 1,8 naar 2,4 (slechter); ongelijke pauzes 15,9 naar 16,2 en bij +3 dB 15,3 naar 15,9 (binnen de meetonzekerheid). De rest is gelijk.

Gewone testbank, 0.15.0 naar 0.15.1: 0 dB 20 WPM 0,7 naar 0,0; 0 dB 28 en 36 WPM 10,1 en 15,8 naar 9,5 en 12,7; -3 dB 20 WPM 38,0 naar 33,7; -3 dB 36 WPM 68,3 naar 66,2; -6 dB 12 WPM 62,9 naar 64,0; diepe QSB 34,5 naar 34,8 en bij +3 dB 43,6 naar 44,3. De rest is gelijk. Let op: de testberichten bestaan grotendeels uit woorden die in de woordenlijst staan, dus de winst bij verloren elementen is daar optimistisch.

Opnames (tekst niet bevestigd):

| Opname | 0.15.0 | 0.15.1 |
|---|---|---|
| 3552,1 kHz | `Q CQ CQ DE IK1QHB ITT1QHB IK1QHB PSE K ME` | `Q CQ CQ DE IK1QHB IK1QHB IK1QHB PSE K ME` |
| 7011,7 kHz | `CQ CQ DE RM IM Y R M3Y CQ CQ CQ DE EN M3Y ENM MY U N K` | `CQ CQ DE RMIMY RM3Y CQ CQ CQ DE RM3Y ENM MY IT N K` |
| 7026,8 kHz, toon vast op 416 Hz | `T NEW LAPT OP 9EE COS TS REEN RI + DI D / FAIA ED N TH OLD ONE = = WE DT A WENTINTO NORWICH TO GETIT CHECKED OUT BUT ONLYFIX WUD EE TO IUT` | `T NEW LAPTOP 9EE COS TS REEN RI + DID / FAIA ED N TH OLD ONE = = WE DT A WENT INTO NORWICH TO GET IT CHECKED OUT BUT ONLY FIX WUD EE TO IUT` |

De andere opnames van 10 november zijn gelijk of verschillen één teken.

### Wat niet opgelost is

In de opname van 7026,8 kHz vallen op meerdere plaatsen twee of meer elementen na elkaar weg (`TS REEN` voor waarschijnlijk SCREEN, `FAIA ED` voor FAILED, `N TH` voor ON THE, `WUD EE` voor WUD BE). Er wordt maar één verloren element per woord teruggezet. Meer terugzetten zonder bewijs zou tekst verzinnen. Dit vraagt een andere aanpak: de decoder zelf laten weten waar het signaal even wegviel (het niveau van de toon volgen binnen een woord), zodat een gat als "onbekend" telt en niet als stilte.

## Zwakke elementen in snelle fading (0.16.0)

Aanleiding: in de opname van 7026,8 kHz (11 november) las de decoder `TS REEN`, `FAIA ED`, `N TH OLD ONE`. De omhullende van die opname toont wat er gebeurt: het niveau van de toon schommelt binnen een halve seconde tussen 27 en 17 dB boven de ruis (per halve seconde: 28 28 26 24 12 19 17 27 29 28 18 17 8 23 26). De punten en strepen in zo'n dal zijn er wel, op 14 tot 19 dB boven de ruis, maar ze telden als stilte omdat ze ver onder het gevolgde niveau zaten. Er zijn dus geen "gaten" zonder signaal: de decoder keek te hoog.

Tegelijk zit er in die opname een tweede, zwakker station op 361 Hz (53 Hz lager, 10 tot 13 dB boven de ruis). Wie de drempel gewoon verlaagt, leest dat station mee.

### Wat er veranderd is (voorverwerking, `frontend.rs`)

Een frame telt nu ook als "sleutel neer" als het zwakker is dan het gevolgde niveau, op vier voorwaarden:

1. Het staat minstens vier ruisspreidingen boven de ruis, gemeten naast de toon (niet uit de onderste groep van de omhullende, want daar zitten bij diepe fading de zwakke elementen zelf in).
2. Het is niet meer dan 12 dB zwakker dan het gevolgde niveau. Elke halvering van het niveau kost 0,5 nat.
3. Het zit binnen 15 Hz van de toon, gemeten uit de fasestap tussen frames. Een station ernaast valt hierdoor af.
4. Het houdt zijn niveau: binnen een halve eenheid ervoor en erna staat niets dat meer dan twee keer zo sterk is. Zonder deze voorwaarde telden de flanken van sterke elementen mee en liepen korte pauzes dicht (lichte weging ging van 0 naar 6 procent fouten). Hiervoor wordt elk frame een halve eenheid vastgehouden (15 ms bij 20 WPM).

Daarnaast: bij het opnieuw afspelen na een toonwissel worden de vastgehouden frames weggegooid (anders stond er een losse E voor de tekst).

### Metingen (0.15.1 naar 0.16.0)

Opname 7026,8 kHz, toon vast op 416 Hz:

- 0.15.1: `T NEW LAPTOP 9EE COS TS REEN RI + DID / FAIA ED N TH OLD ONE = = WE DT A WENT INTO NORWICH TO GET IT CHECKED OUT BUT ONLY FIX WUD EE TO IUT`
- 0.16.0: `T NEW LAPTOP 9EE COS TS I RREEN R I + FA DID / FAILED ON TH OLD ONE = = WE DT A WENT INTO N ORWICH TO GET IT CHECKED OUT BUT ONLY FIX WUD IE TO NUT`

Beter: `FAILED`, `ON TH`. Niet beter: `TS I RREEN` (waarschijnlijk THE SCREEN), `WUD IE`. Slechter: `N ORWICH`.

Opnames van 10 november: 7005,0 kHz leest nu `NICE TO MEET YOU MY FRIEND` (was `MEET YE U`), 3552,1 kHz eindigt op `PSE K` zonder losse tekens erna, 7009,0 kHz `LARRY LARRY`. 7011,7 kHz is iets slechter (`EN M3Y ENM3Y`, was `RM3Y`), 7008,9 kHz wisselt (`THORNY` goed, `S:RYLOTSOR` slechter).

Testbank, cellen die veranderden:

| Geval | 0.15.1 | 0.16.0 |
|---|---|---|
| Diepe QSB, +10 dB | 34,8 | 31,4 |
| Diepe QSB, +3 dB | 44,3 | 43,2 |
| 0 dB, 28 WPM | 9,5 | 6,8 |
| 0 dB, 36 WPM | 12,7 | 9,2 |
| 0 dB, 20 WPM | 0,0 | 1,0 |
| +3 dB, 36 WPM | 0,7 | 1,4 |
| -3 dB, 12 / 20 / 36 WPM | 13,6 / 33,7 / 66,2 | 12,5 / 36,3 / 75,0 |
| -6 dB, 12 / 20 / 28 WPM | 64,0 / 90,1 / 94,6 | 68,8 / 86,1 / 92,3 |
| 15 procent jitter | 2,4 | 1,7 |
| Klikken 40 per s | 0,3 | 1,0 |
| Draaggolf 120 Hz verder | 6,1 | 4,7 |
| Draaggolf 50 Hz, start samen | 14,2 | 16,6 |
| Seinsleutel (pauzes 30, elementen 15 procent) | 19,8 | 23,9 |
| Ongelijke pauzes 30 procent bij +3 dB | 15,9 | 18,0 |
| Tekens aan elkaar | 0,0 | 0,6 |

Tweede station 60 Hz verder op gelijk niveau: 0,0 in beide (zonder voorwaarde 3 was dat 4,5 tot 16 procent). `seekbench` is gelijk aan 0.15.1.

Eerlijk beeld: winst bij fading en bij 0 dB op hoge snelheid, verlies bij -3 dB op 36 WPM, bij -6 dB op 12 WPM en bij de slordigste vuisten. Het zijn geen grote verschuivingen, behalve -3 dB op 36 WPM (66 naar 75). Wie liever het oude gedrag heeft: `weak_floor_sd = 0` in `FrontendConfig` zet het uit (nog geen instelling in de app).

### Paddle oefenen

Vraag van ON3VZ: reageert de oefenmodus nu op de paddle? De fout uit 0.14.0 (detector luisterde op de decodertoon) is hersteld in 0.15.0 en getest op de testbank, nog niet bevestigd op de radio (test A46). In 0.16.0 start een paddle-oefening bovendien zelf het luisteren naar de radio als dat nog niet liep: stond de audio uit, dan kwam er alleen een melding en gebeurde er niets.

## Schakelaar Fading (0.16.1)

Op vraag van ON3VZ staat het gedrag van 0.16.0 achter een schakelaar op het hoofdscherm (kaart Audio): "Doorlezen" (standaard, zwakke elementen in een fade tellen mee) en "Streng" (zoals tot en met 0.15.1). De tabel in het vorige hoofdstuk is precies het verschil tussen beide standen: de kolom 0.15.1 is "Streng", de kolom 0.16.0 is "Doorlezen".

Extra meting voor het geval waarvoor de schakelaar bedoeld is: een zuiver signaal (+25 dB) waarvan het niveau ongeveer één keer per seconde 10 dB zakt (`qsb_depth` 0,67, `qsb_hz` 1,1), 6 berichten van 22 tekens. "Doorlezen": 1 fout teken op 132. "Streng": 83 op 132. De trage fading in de gewone testbank (0,3 Hz) toont dit verschil veel minder (34,8 tegen 31,4 procent), omdat het signaal daar in de dalen echt in de ruis verdwijnt.

## Zender die van tempo of spatiëring wisselt (0.17.0)

Aanleiding: een zender die een naam traag spelt en daarna gewoon verder seint, of omgekeerd. De decoder volgde de spatiëring traag, zodat na de wissel tekens samensmolten of een gespeld woord als losse letters kwam.

### Nieuwe gevallen in de testbank

De generator kan na een deel van de tekst overschakelen op andere pauzes en een andere snelheid (`change_after`, `char_gap2`, `word_gap2`, `wpm2`). Voor deze gevallen worden drie berichten na elkaar geseind, zodat er na de wissel genoeg tekst volgt. 20 WPM, +15 dB, 12 uitzendingen per geval. CER in procent, 0.16.1 en 0.17.0:

| Geval | 0.16.1 | 0.17.0 |
|---|---|---|
| Spelt traag (teken 8, woord 18), seint daarna gewoon | 3,1 | 2,6 |
| Seint gewoon, spelt daarna traag (teken 8, woord 18) | 6,0 | 3,0 |
| Ruim (teken 6, woord 13), daarna krap (teken 2,5, woord 5,5) | 10,5 | 5,1 |
| Snelheid van 18 naar 28 WPM | 0,0 | 0,0 |
| Snelheid van 30 naar 18 WPM | 0,0 | 0,0 |
| Spelt traag op 25 WPM, daarna krap op 32 WPM | 19,9 | 8,3 |
| Beginner met lange pauzes, Farnsworth | 2,4 | 0,9 |

Een wissel van alleen de snelheid ging al goed. Het probleem zat bij de spatiëring.

### Wat er veranderd is

1. **Van ruim naar krap** (`Decoder::tightened`). Na ruim seinen ligt de grens tussen "pauze binnen een teken" en "pauze tussen tekens" hoog. Seint de zender dan krap, dan vallen zijn nieuwe tekenpauzes onder die grens en smelten tekens samen. Dat is te zien aan de pauzes zelf: ze vallen in twee groepen, rond één eenheid en een tweede groep duidelijk hoger. Als die tweede groep duidelijk korter is dan de tekenpauze die de decoder tot dan aannam, wordt ze meteen de nieuwe tekenpauze. Alleen bij een zender die als ruim geleerd was (factor 1,3 of meer), anders gaf het valse meldingen bij heel ongelijk seinen.
2. **Woordgrenzen bij die wissel** (`spacing.rs`). Wat nog vastgehouden werd van voor de wissel wordt met de oude drempel afgehandeld, de rest met een drempel voor de nieuwe spatiëring.
3. **Van gewoon naar traag spellen.** Vier of meer pauzes na elkaar van ongeveer dezelfde lengte zijn tekenpauzes, hoe lang ze ook zijn: niemand seint drie woorden van één letter na elkaar. Het eerste gespelde woord kwam anders als losse letters.

### Controle

De gewone testbank (alle cellen) is identiek aan 0.16.1. De andere vuisten zijn gelijk, behalve 6 op 100 elementen verloren (20,1 naar 20,6). `seekbench` is gelijk.

De acht echte opnames lezen **precies hetzelfde** als met 0.16.1. Ook de opname van 7008,9 kHz, die de aanleiding was, werd dus niet beter: de fouten in het snelle stuk daar (`S:RY` voor SORRY) blijken niet van samengesmolten tekens door de wissel te komen. In die opname sloeg de nieuwe herkenning pas helemaal op het einde aan. Zonder de juiste tekst kan ik niet zeggen wat daar wel misgaat.

## Clusterspots als hint voor de decoder (0.18.1)

Vraag: helpt het als de decoder de roepletters kent van spots vlak bij de frequentie, en kan een foute spot schade doen?

Meting met `cargo run --release -p cwld-decoder --example hintbench`: 60 berichten per geval, telkens met een roepletter die de decoder nooit zag (80 roepletters per geval). Drie keer gedecodeerd: zonder hints, met de juiste roepletter tussen drie foute, en met alleen foute hints. De foute hints zijn bewust lastig: de echte roepletter met één teken anders, de echte roepletter met één punt of streep meer, en twee willekeurige.

| Geval | Zonder hints: fout %, roepletter goed % | Juiste hint | Alleen foute hints | Foute hint in tekst |
|---|---|---|---|---|
| Schoon, +15 dB | 0,0 / 100 | 0,0 / 100 | 0,0 / 100 | 0 |
| 3 op 100 elementen weg | 7,3 / 60,0 | 4,9 / 85,0 | 7,3 / 60,0 | 0 |
| 6 op 100 weg, +6 dB | 17,2 / 38,8 | 14,4 / 67,5 | 17,2 / 38,8 | 0 |
| Slordig handwerk | 25,9 / 23,8 | 24,0 / 40,0 | 25,9 / 23,8 | 1 |
| Zwak, 0 dB | 3,3 / 85,0 | 2,6 / 91,2 | 3,3 / 85,0 | 0 |
| Diepe QSB | 48,7 / 0,0 | 48,7 / 0,0 | 48,7 / 0,0 | 0 |

Wat dit zegt:

- Een juiste hint helpt waar elementen wegvallen of pauzes verkeerd vallen. De roepletter zelf komt duidelijk vaker goed door.
- Bij diepe QSB helpt het niet: daar vallen meerdere elementen na elkaar weg en dat herstelt de decoder niet.
- Foute hints veranderden de tekst in geen enkel geval meetbaar. Eén keer op 480 roepletters kwam een foute hint in de tekst.
- Dit is synthetisch. Op de band is het nog niet gemeten (test A53). Een skimmer kan ook zelf een foute roepletter melden; dat geval zit in de foute hints, maar echte fouten van skimmers kunnen anders zijn.

## Klein neuraal netwerk als extra bron (0.18.3, niet ingeschakeld)

Vraag: levert een klein netwerk, getraind op de CPU, betere waarschijnlijkheden op of de sleutel neer is, vooral bij zwakke signalen en QSB?

Opzet:

- Netwerk met 6001 gewichten (99 ingangen, lagen van 48 en 24). Per frame van 5 ms kijkt het naar de omhullende en de klassieke waarschijnlijkheid van 120 ms aan elke kant, geschaald op het signaalniveau van de laatste twee seconden.
- Training op 700 synthetische berichten van de eigen generator (12 tot 38 WPM, -8 tot +15 dB, met en zonder QSB, ongelijke pauzes), controle op 150 andere. Trainen duurt minder dan een minuut op twee CPU-kernen, met alleen numpy (`tools/nn/train.py`).
- Uitvoeren in gewone Rust, zonder bibliotheek. Het antwoord komt 240 ms later, omdat het netwerk vooruit kijkt.

Meting per frame (controleset): het netwerk raadt de sleutelstand veel vaker goed dan het klassieke getal per frame (10,0 tegen 19,9 procent fout). Dat is geen eerlijke vergelijking: het klassieke getal kijkt naar één frame, en het zoekalgoritme van de decoder voegt de samenhang in de tijd daarna zelf toe.

Meting door de hele decoder (volledige testbank, 71 cellen, tekenfouten in procent):

| Variant | Gemiddeld | Beter | Slechter |
|---|---|---|---|
| Klassiek (0.18.2) | 10,7 | | |
| Netwerk opgeteld bij klassiek (sterkte 0,2) | 10,8 | 10 cellen | 19 cellen |
| Netwerk in plaats van klassiek (snelle testbank) | 11,9 tegen 10,3 | 6 | 24 |

Enkele rijen, klassiek tegen opgeteld: 0 dB 0,4 / 1,0 / 6,8 / 9,2 tegen 0,7 / 0,0 / 7,4 / 13,4 (12, 20, 28, 36 WPM); -3 dB 12,5 / 36,3 / 57,4 / 75,0 tegen 14,0 / 32,7 / 58,0 / 75,4; diepe QSB 31,4 tegen 34,1; draaggolf op 50 Hz die samen met het signaal start 16,6 tegen 13,5.

Besluit: niet inschakelen. Er is geen winst, en de verschillen per cel vallen binnen de spreiding van de testbank. De reden is waarschijnlijk dat het tijdsmodel van de decoder (Viterbi met duurkansen) de informatie uit de omgeving al benut; het netwerk voegt hetzelfde nog eens toe. Als vervanging is het slechter bij hoge snelheid, omdat het de randen van korte punten uitsmeert.

Wat wel kans maakt, later: een netwerk dat rechtstreeks tekens leest (met CTC) in plaats van de sleutelstand per frame, of afstellen op echte opnames waarvan de juiste tekst bekend is. Het eerste is veel groter en past niet bij "trainen in een minuut"; het tweede vraagt opnames met tekst. De code en het trainingsgereedschap blijven in de repository, uitgeschakeld (`PipelineConfig::neural`, in de testbank met `CWLD_NN=-0.2`).

