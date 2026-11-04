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
