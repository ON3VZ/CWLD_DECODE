# CWLD

> Repository: `CWLD_DECODE`. The app itself is called **CWLD**.

A modern, simple, open-source CW station app for Windows and Linux (macOS later). It is built to control an Icom IC-7300 MK II and a QRP Labs QMX, decode CW, send CW from the keyboard and clickable chips next to your paddle, and work together with Log4OM.

Status: **phase 2 (radio and sending)**. Drivers for the IC-7300 MK II (CI-V) and the QMX (CAT) are written and tested against a fake serial port, but **not yet on real radios**: see `docs/testplan_radio.md`. There is no audio or decoder yet. See `CHANGELOG.md`.

## Three modes

| Mode | Transmits? | Meaning |
|---|---|---|
| LIVE | Yes | Sending reaches the radio. Needs a confirmation. CWLD never starts in this mode. |
| LISTEN | Never | Decode only. The radio can be in any mode. What you type is not sent. |
| PRACTICE | Never | Sending is simulated with a sidetone, nothing leaves the application. |

Only LIVE can reach the radio: the rule sits in one place (`GuardedRig`) and is covered by tests.

## Build and test

Requirements: Rust (stable), Node 20 or newer. On Linux also the Tauri system libraries (webkit2gtk 4.1, gtk 3, libsoup 3, librsvg, openssl, alsa).

```
cargo test --workspace --exclude cwld   # Rust tests
cd ui && npm install && npm test        # interface tests
npm run check && npm run build          # type check and build
cargo tauri dev                         # run the app (needs tauri-cli)
```

`npm run dev` in `ui/` runs the interface in a browser with a mock of the core (same rules), handy for design work.

## Layout

- `crates/cwld-types`: shared types (policy, commands, settings, Morse timing)
- `crates/cwld-radio`: `Rig` trait, transmit guard, simulator, keyer, Icom CI-V and QMX CAT drivers
- `crates/cwld-core`: application logic, no window needed
- `src-tauri`: thin Tauri shell
- `ui`: Svelte 5 interface, texts in `ui/src/i18n/en.json` and `nl.json`

## License

GPL-3.0-or-later. No code from other CW programs is used.

---

# CWLD (Nederlands)

Een moderne, eenvoudige CW-station-app met open source voor Windows en Linux (macOS later). Ze is gemaakt om een Icom IC-7300 MK II en een QRP Labs QMX te bedienen, CW te decoderen, CW te zenden met het toetsenbord en klikbare chips naast je paddle, en samen te werken met Log4OM.

Stand van zaken: **fase 2 (radio en zenden)**. Stuurprogramma's voor de IC-7300 MK II (CI-V) en de QMX (CAT) zijn geschreven en getest met een nep-seriele poort, maar **nog niet op echte radio's**: zie `docs/testplan_radio.md`. Er is nog geen audio of decoder. Zie `CHANGELOG.md`.

## Drie modi

| Modus | Zendt? | Betekenis |
|---|---|---|
| LIVE | Ja | Zenden bereikt de radio. Vraagt bevestiging. CWLD start nooit in deze modus. |
| LUISTEREN | Nooit | Alleen decoderen. De radio mag in elke modus staan. Wat je typt wordt niet verzonden. |
| OEFENEN | Nooit | Zenden wordt gesimuleerd met een meeluistertoon, niets verlaat de toepassing. |

Alleen LIVE kan de radio bereiken: de regel zit op één plaats (`GuardedRig`) en is met tests afgedekt.

## Bouwen en testen

Zie de Engelse tekst hierboven, de commando's zijn dezelfde.
