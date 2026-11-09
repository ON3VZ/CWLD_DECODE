# CWLD ontwerp: wedstrijdprofielen (Ronde C1, voorstel ter goedkeuring)

Stand 7 november 2026. Kristof heeft het ontwerp goedgekeurd ("denk dat goed is") en de vragen in hoofdstuk 9 beantwoord. Er is nog niets gebouwd. De bouw begint na een expliciete GO.

## 1. Doel

Een wedstrijd kiezen met één klik en meteen de juiste uitwisseling kunnen zenden, zonder per wedstrijd iets te programmeren. Er zijn meerdere profielen. Kristof kan ze kopiëren en zelf nieuwe maken.

## 2. Wat een profiel is

Een profiel bestaat uit:

- **Naam** (bv. "UBA DX Contest CW") en een korte **opmerking** die bij het kiezen verschijnt (bv. bij CQ WW: "Het gebruik van een CW-decoder geldt als hulpmiddel. Schrijf je in de klasse met hulp in als je CWLD gebruikt.").
- **Oproepzinnen** (bv. `TEST UBA`, `CQ ON Contest`). Elke wordt een chip.
- **Uitwisseling die jij zendt**: één of meer sjablonen met een label, bv. "Vanuit België" = `{CALL} {RST} {NR} {SEC}` en "Buiten België" = `{CALL} {RST} {NR}`. Je kiest met een keuzelijst welk sjabloon actief is.
- **Eén extra veld dat je ontvangt** (of geen): naam, soort (tekst, getal of keuze uit een lijst) en wanneer het geldt: altijd, of alleen als het andere station een bepaald voorvoegsel heeft (bv. ON, OO, OP, OQ, OR, OS, OT).
- **Volgnummer gebruiken**: ja of nee (bepaalt of `{NR}` telt).
- **Verkorte cijfers**: uit, of aan voor het volgnummer (001 wordt TT1, 019 wordt T1N). Het rapport blijft zoals ingesteld (5NN of 599).

## 3. Meegeleverde profielen

Alleen-lezen, ter controle tegen het actuele reglement. Elk heeft een knop **Kopiëren** om er een eigen versie van te maken, en **Standaard herstellen** voor een meegeleverd profiel dat Kristof heeft aangepast.

| Profiel | Zendt (vanuit België) | Extra veld dat je ontvangt |
|---|---|---|
| UBA DX Contest CW | `{CALL} {RST} {NR} {SEC}` | UBA-sectie, alleen bij ON-stations |
| ON Contest CW | `{CALL} {RST} {NR} {SEC}` | UBA-sectie, alleen bij ON-stations |
| UBA Spring Contest CW | `{CALL} {RST} {NR} {SEC}` | UBA-sectie, alleen bij ON-stations |
| UBA Low Band Winter CW | `{CALL} {RST} {SEC}` (geen nummer) | geen |
| CQ WW DX CW | `{CALL} {RST} {ZONE}` | CQ-zone |
| CQ WPX CW | `{CALL} {RST} {NR}` | geen (het volgnummer wordt door de log bijgehouden) |
| ARRL International DX CW | `{CALL} {RST} {PWR}` | staat of provincie (alleen W en VE) |
| IARU HF Championship | `{CALL} {RST} {ITU}` | ITU-zone of HQ-afkorting |
| WAE DX Contest CW | `{CALL} {RST} {NR}` | geen (000 als het andere station geen nummer zendt) |
| Leeg sjabloon | leeg | zelf in te vullen |

Bronnen: de officiële reglementen (zie ook claude/ontwerp_architectuur.md en de lijst onderaan). Winter Contest en Spring Contest komen van oudere pagina's en worden bij de bouw opnieuw nagekeken. CQ 160 volgt als Kristof dat wil. QTC's van WAE komen later.

## 4. Eigen variabelen

In Instellingen, rubriek Contest, een tabel "Eigen variabelen":

- Naam (hoofdletters, cijfers en onderstrepingsteken, bv. `SEC`, `ZONE`, `ITU`, `PWR`, `AGE`).
- Waarde (bv. `WLD` voor `{SEC}`).
- Gereserveerde namen kunnen niet gebruikt worden (CALL, MYCALL, NAME, RST, QTH, NR).
- Ontbreekt een waarde die in het gekozen sjabloon voorkomt, dan toont CWLD een waarschuwing vóór het zenden en verzendt het niets onvolledigs. Dit gebruikt dezelfde regel als nu al: ontbrekende variabelen worden gemeld, niet geraden.

Standaardwaarden (Kristof, 7 november): `{SEC}` = `WLD`, `{ZONE}` = `14` (CQ-zone van België), `{ITU}` = `27` (ITU-zone van België), `{PWR}` = `20` (watt). De zones zijn gecontroleerd op drie pagina's (qsl.net landenlijsten en een QRZCQ-oproepsignaalpagina); de reglementen zelf noemen ze niet. Alles is aanpasbaar.

## 5. Schermen

**Hoofdscherm.** Naast de knop Contest staat een keuzelijst met het actieve profiel. Bij het kiezen verschijnt de opmerking van het profiel één keer. De chips bevatten de oproepzinnen en de uitwisseling van het actieve sjabloon. Een kleine keuzelijst toont het actieve sjabloon als er meer dan één is.

**Instellingen, rubriek Contest.**
- Links de lijst met profielen met knoppen Nieuw, Kopiëren, Verwijderen.
- Rechts de editor: naam, opmerking, oproepzinnen, sjablonen met een **voorbeeld** onder elk sjabloon (met `PA3ABC` als voorbeeldstation en echte waarden uit je variabelen), extra veld, verkorte cijfers.
- Daaronder de tabel Eigen variabelen.
- Elke instelling heeft een korte uitleg en Standaard herstellen, zoals de rest van de app.

**Reservekopie en delen.** Knoppen Exporteren en Importeren (JSON-bestand) voor één profiel of voor alle profielen en eigen variabelen samen. Zo heb je een reservekopie en kun je een profiel delen met de club. Importeren vraagt wat er gebeurt bij een profiel met dezelfde naam: overschrijven of een kopie maken.

## 6. Hoe het in de code past

- Instellingen (Rust, `settings.json`): nieuw blok `contest` met `active_profile`, `profiles` en `vars`. De bestaande waarden `contest_mode` en `contest_nr` blijven werken, dus oude instellingenbestanden laden zonder fout (migratietest).
- Sjablonen: de bestaande `expand()` in `ui/src/lib/template.ts` blijft de enige plek die `{NAME}` vervangt. Eigen variabelen komen daar als extra waarden bij. Verkorte cijfers zijn een kleine functie op `{NR}`.
- Chips: de bestaande chips met `{NR}` en de Exchange-chip worden gegenereerd uit het actieve profiel.
- Alles in het Engels en Nederlands (en.json, nl.json).
- Tests: expansie met eigen variabelen, ontbrekende variabelen, verkorte cijfers, migratie van oude instellingen, regel voor het voorvoegsel, en de meegeleverde profielen (elk profiel moet zonder ontbrekende gereserveerde variabelen kunnen worden uitgewerkt).

## 7. Wat hier niet in zit (komt met het logboek, C2 en C4)

- Het ontvangen extra veld invullen en bewaren: dat heeft het logboek nodig.
- Volgnummer oplopen bij Loggen, dupewaarschuwing, export (ADIF, later Cabrillo).
- Lijst met UBA-sectiecodes met aanvullen bij typen (bron: officieel reglement, met Kristof controleren).

## 8. Bouwvolgorde

1. **C-1 (Sonnet), zonder deadline:** instellingenmodel, meegeleverde profielen, eigen variabelen, editor, chips en keuzelijst, verkorte cijfers, tests, docs. Dat is de zendkant. De zip volgt na tests.
2. **C-2 (Sonnet):** logboek, export, het ontvangen extra veld, het logpaneel, dupewaarschuwing, volgnummer bij Loggen, sectielijst.

## 9. Besluiten van Kristof (7 november)

1. Ontwerp goedgekeurd.
2. CQ 160 Contest: niet nodig.
3. C-1 hoeft niet klaar te zijn voor CQ WW CW (28 en 29 november): geen haast.
4. CQ-zone en ITU-zone zoekt Claude op (14 en 27, zie hoofdstuk 4). Vermogen 20 watt.
5. Profielen exporteren: ja, als reservekopie. Importeren hoort erbij, anders is een reservekopie niet terug te zetten.

## 10. Bronnen

- UBA DX Contest: https://www.uba.be/en/hf/contest-rules/uba-dx-contest
- UBA ON Contest: https://www.uba.be/en/hf/contest-rules/on-contest
- UBA Spring Contest (2023): https://old.uba.be/en/hf/contest-rules/spring-contest
- UBA Low Band Winter Contest (2014): https://old.uba.be/en/hf/contest-rules/low-band-winter-contest
- CQ WW DX Contest 2026: https://cqww.com/rules/2026_rules_cqww.pdf
- CQ WPX Contest 2025: https://www.cqwpx.com/rules/2025_cqwpx_rules.pdf
- ARRL International DX Contest: https://contests.arrl.org/ContestRules/DX-Rules.pdf
- IARU HF Championship: https://contests.arrl.org/ContestRules/IARU-HF-Rules.pdf
- WAE DX Contest 2025: https://www.darc.de/der-club/referate/conteste/wae-dx-contest/en/wae-rules/
