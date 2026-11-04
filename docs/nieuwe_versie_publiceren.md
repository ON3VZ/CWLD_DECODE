# Een nieuwe versie van CWLD publiceren

Als dit één keer is ingesteld, krijgen alle installaties van CWLD de nieuwe versie vanzelf aangeboden.

## Eenmalig: de updatesleutel in GitHub zetten

De sleutel ondertekent elke update. Zonder deze sleutel weigert de app een update. Je hebt twee dingen nodig: het bestand `cwld-updater.key` en het wachtwoord (kreeg je in de chat). Bewaar beide ook op een veilige plek buiten de computer (bv. wachtwoordmanager). Raak je ze kwijt, dan kunnen bestaande installaties nooit meer bijwerken en moet iedereen opnieuw installeren.

1. Open https://github.com/ON3VZ/CWLD_DECODE in je browser.
2. Klik bovenaan op **Settings**.
3. Klik links op **Secrets and variables**, dan op **Actions**.
4. Klik op **New repository secret**.
5. Naam: `TAURI_SIGNING_PRIVATE_KEY`. Bij Secret plak je de volledige inhoud van `cwld-updater.key` (open het bestand met Kladblok, Ctrl+A, Ctrl+C). Klik **Add secret**.
6. Klik opnieuw **New repository secret**. Naam: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Secret: het wachtwoord. Klik **Add secret**.
7. Ga naar **Settings**, **Actions**, **General**, scroll naar **Workflow permissions**, kies **Read and write permissions** en klik **Save**. Dit laat de workflow een Release aanmaken.

## Elke nieuwe versie

1. Verhoog het versienummer in `src-tauri/tauri.conf.json` (bv. 0.3.8) en zet bovenaan `CHANGELOG.md` een kop `## 0.3.8 (korte titel)` met wat nieuw is. Die tekst ziet de gebruiker bij de update.
2. Open GitHub Desktop, schrijf links onder Summary een korte tekst, klik **Commit to main** en dan **Push origin**.
3. Klik in het menu op **Repository**, dan **Create tag...**. Typ `v0.3.8` (met de v, zelfde nummer als in stap 1) en klik **Create tag**.
4. Klik op **Push origin**. De tag gaat nu mee naar GitHub.
5. Open op GitHub het tabblad **Actions**. De workflow **release** start. Na ongeveer 10 tot 15 minuten staat de nieuwe versie onder **Releases** (rechts op de hoofdpagina), met het installatiebestand en `latest.json`.
6. Start CWLD op een computer met een oudere versie: bovenaan verschijnt "Nieuwe versie 0.3.8 beschikbaar". Klik **Bijwerken**.

## Eerste keer

De versie die je nu hebt (0.3.6) bevat nog geen updatefunctie. Installeer 0.3.7 eenmalig met de hand (de installer uit de release). Vanaf dan kan de app zelf bijwerken. Test het met een tweede versie, 0.3.8.

## Als er iets misgaat

- **Workflow rood bij "Run tauri-action", melding over signing key:** de secret mist of staat verkeerd. Maak hem opnieuw aan (stap 5 en 6).
- **App zegt "Zoeken lukt niet":** controleer dat er een Release bestaat die niet als Draft of Pre-release staat en dat `latest.json` erin zit.
- **Windows SmartScreen** blijft melden dat de app van een onbekende uitgever is, tot er een Windows-codesignatuur is. Dat staat los van de updatesleutel. Klik Meer info, dan Toch uitvoeren.
