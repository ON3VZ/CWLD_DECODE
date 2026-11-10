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

