# CWLD installeren op Windows (testversie)

Bij elke upload naar GitHub bouwt GitHub zelf een Windows-installer. Je hoeft niets te compileren.

## De installer downloaden

1. Ga naar https://github.com/ON3VZ/CWLD_DECODE en klik bovenaan op het tabblad **Actions**.
2. Klik in de lijst op de nieuwste run met de naam **windows-build** (groen vinkje = gelukt).
3. Scrol helemaal naar beneden naar het kopje **Artifacts** en klik op **CWLD-windows-installer**. Er wordt een zip gedownload.
4. Klik rechts op de zip in Verkenner en kies **Alles uitpakken**. Daarin zit een bestand `CWLD_..._x64-setup.exe`.

## Installeren

1. Dubbelklik op het `.exe`-bestand.
2. Windows toont "Windows heeft uw pc beschermd" (SmartScreen), omdat de app nog niet digitaal ondertekend is. Klik op **Meer info** en daarna op **Toch uitvoeren**.
3. Volg de installatie. CWLD staat daarna in het Startmenu.

## Eerste test

1. Zet de IC-7300 MK II aan en verbind hem met de USB-C kabel.
2. Start CWLD en open de instellingen (tandwiel). Kies de radio en de audio-ingang.
3. Volg daarna `docs/testplan_radio.md`, te beginnen met de audiotests A1 tot A10.
4. Maak met de REC-knop opnames van een zuiver signaal, een zwak signaal en een signaal met storing. Die gebruiken we om de decoder af te stellen.

Als de bouw rood is (kruisje), open de run, klik op de rode stap en stuur me de laatste regels uit de foutmelding.
