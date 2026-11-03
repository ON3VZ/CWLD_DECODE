# CWLD decoder: eerste meetresultaten (fase 3, tussenstand)

Let op: dit zijn metingen op **synthetische CW met witte ruis**. Ze zeggen nog niets over echte banden en nog niets over MRP40 of CwGet. Die vergelijking volgt met echte opnames van Kristofs radio (testplan I10 en Q10).

## Wat gemeten is

Tekenfoutpercentage (CER): bewerkingsafstand tussen de gezonden en de gedecodeerde tekst, gedeeld door de lengte van de gezonden tekst. Spaties tellen mee. SNR is het vermogen van de toon bij neergedrukte sleutel tegenover de ruis in 500 Hz. Per cel 4 berichten uit een vaste set van 10 QSO-zinnen, handsleutelfout 5 procent. Herhalen: `cargo run --release -p cwld-decoder --bin cwld_bench -- --quick`.

| SNR dB | 12 WPM | 20 WPM | 28 WPM | 36 WPM |
|---|---|---|---|---|
| +30 tot +3 | 0,0 | 0,0 | 0,0 | 0,0 |
| 0 | 2,4 | 4,1 | 4,2 | 16,7 |
| -3 | 21,2 | 57,1 | 59,2 | 74,4 |
| -6 | 176,5 | 126,5 | 87,5 | 86,7 |

Moeilijke signalen (20 WPM, +10 dB): handsleuteling 15 procent 0,9 ; slordige spatiering x1,5 0,9 ; korte en lange strepen 0,0 ; toonafwijking 25 Hz 0,0 ; toondrift 20 Hz 0,0 ; tweede station op 60 Hz even sterk 0,0 ; tweede station +10 dB op 120 Hz 0,0 ; 8 WPM bij 0 dB 0,0 ; 45 WPM bij 15 dB 0,0 ; **diepe fading (QSB 0,8) 39,6**.

## Wat het betekent

* Vanaf +3 dB is de decoder op dit testmateriaal foutloos, ook bij slordige sleutel, afwijkende strepen en een tweede station ernaast.
* Rond 0 dB begint het te haperen, vooral bij hoge snelheid.
* Onder -3 dB verzint de decoder tekens uit ruis (CER boven 100 procent betekent meer fouten dan tekens). Dit is de eerste verbetering die nodig is.
* Diepe fading is zwak: de signaalniveau-schatting volgt de snelle dalen niet.

## Bekende beperkingen

Dit is een eerste versie. Er is nog geen taalmodel (woordenlijst, roepletters), geen beam search met meerdere woordhypotheses, geen meerdere signalen tegelijk, en de koppeling met de audio-ingang en de interface moet nog gebouwd worden. Er is nog geen kwaliteitsdoel vastgelegd: dat komt pas na metingen met echte opnames.

## Wat nog moet

1. Onder -3 dB minder valse tekens: betere evidentie en een gokken-verbod bij lage zekerheid.
2. Fading: snellere niveauvolging.
3. Taalmodel en roepletters (stap 5 van het plan).
4. Echte opnames als testmateriaal en een eerlijke vergelijking met CwGet via WAV.
5. Audio-ingang (cpal) en koppeling met de waterfall en het gesprek.
