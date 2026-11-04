## 0.4.4 (installer no longer stuck on "Failed to kill CWLD")

- Fixed (to be confirmed on Windows): the installer and the uninstaller stopped with "Failed to kill CWLD. Please close it first then try again" even after CWLD was closed. Cause: the WebView2 processes that draw the CWLD window ("WebView2: CWLD" in Task Manager) stayed alive after CWLD closed, and when one was ended by hand, WebView2 started it again. The installer asks Windows Restart Manager which processes still hold cwld.exe, and these leftovers counted.
- The installer and the uninstaller now first stop cwld.exe and only the WebView2 processes of CWLD (found by CWLD's own data folder in their command line, so WhatsApp, Lenovo Vantage and other programs are left alone), and only then do their check.
- When CWLD closes, it waits at most 1.5 seconds for the radio port and the sound card, and then ends its process at once, so a USB driver that does not answer can no longer keep cwld.exe alive. The same 1.5 second limit applies just before an update is installed.
- The first version with this installer is 0.4.4 itself, so installing 0.4.4 over an older version may still need the manual route once (handbook section 11).

## 0.4.3 (connects at start, cleaner updates)

- New: CWLD connects to the saved radio by itself when it starts (Settings, Radio, "Connect when CWLD starts", on by default). A radio that is off or a port that is busy does not stop the start: CWLD just shows "not connected" and you press Connect. The simulator is never connected this way.
- Updates: before the new version is installed, CWLD now lets go of the radio port and the sound card, and when the program ends it makes sure the process really is gone. The installer's message "CWLD is running, failed to kill CWLD" came from a process that stayed behind. The most likely reason updates did not work at all: a private repository gives the app a 404 on the update link (make the repository public, see docs/nieuwe_versie_publiceren.md). This is not proven to be the whole cause of the kill message, see the manual route in the handbook (section 8) if it happens again.

## 0.4.2 (no more text out of noise)

- Fixed: on a quiet band or on only noise the decoder showed a stream of letters (`TTTT MTMT EE`) and the certainty meter stood at 94 percent. Cause: the radio's CW filter makes the noise very uneven (on your recording 40 dB stronger below 800 Hz than above 1000 Hz), so the level test took noise for a signal.
- New: a second check on the shape of the signal. Noise through a narrow filter wanders up and down in short blips, keyed CW holds its level for whole dots and dashes. When nearly every blip in the last seconds is shorter than the filter's reaction time, CWLD decodes nothing. Measured on your noise recording, on two other noise recordings and on white noise: 96 to 100 percent short blips. Keyed CW at +3 dB and better: 0 to 40 percent.
- The certainty meter shows "no text" while only noise is heard, instead of the certainty of old text. The signal to noise value goes away as well.
- Measured: no change on signals of +3 dB and better, a little less invented text at -6 dB (52.9 instead of 96.5 percent at 12 WPM). Signals around -3 dB and weaker are decoded no better or worse than before. See docs/meetresultaten_decoder.md.
- After a pause with only noise, a new signal is recognised after about 1 to 2 seconds. The start of it is kept and decoded afterwards (3 seconds are kept).

## 0.4.1 (library of Q-codes and abbreviations)

- New: a slide-out panel with 30 Q-codes, about 50 common CW abbreviations and the prosigns, each with its meaning in English and Dutch. Open it with the Q button next to Send or with F1, close it with Esc. A search box finds a code or a word in the meaning (for example "locatie" finds QTH).
- A click on a code adds it to your message. For a Q-code there is also a link that adds it as a question (QTH?), with the question spelled out.
- Releasing is now automatic: a push to main that raises the version in tauri.conf.json builds and publishes the release by itself (no tag needed).
- Prosigns are added as <AR>, <SK>, <BT> and so on, which the keyer already translates for the radio. CL is shown but cannot be sent by the keyer.

## 0.4.0 (band scope of the radio)

- New: the band scope of the IC-7300 MK II in the app, as on the radio's screen: a spectrum line and a waterfall over the radio's scope span (for example ±25 kHz). A click on a signal tunes the radio there. The red line is the VFO. The data comes over the same USB cable (CI-V command 27: CWLD switches the scope and its data output on after connecting, and the data output off when disconnecting).
- The audio waterfall (what the decoder hears) moved to its own block on the right, under the QSO panel, and can be hidden there. The band scope can be hidden under the scope itself. Both choices are in Settings, View, and are remembered.
- The simulator has a made-up band scope with five stations, for trying it out without a radio.
- Not yet checked on the radio: whether the MK II sends the scope exactly as the CI-V guide describes (11 pieces of 475 points over USB), and how the span is meant in centre mode.

## 0.3.9 (waterfall, speed in pauses, tab wait setting)

- Decoder: during a pause the speed estimate no longer runs away. Before, a 3.5 s silence made it climb to about 110 WPM and the next transmission came out as single letters (`MTMT MMTM`). The speed only follows real characters now.
- A new tab starts after the frequency has stayed put for 5 seconds (setting Conversations, 1 to 30 seconds).

- The waterfall measured its noise level over the whole strip. With a narrow filter in the radio, most of the strip is silent, so the noise inside the filter came out nearly white. The level is now followed per frequency (a running median per bin, with the median of the neighbours about 80 Hz either side), so noise stays dark whatever the filter shape and real signals stand out. On simulated noise inside a 200 to 900 Hz filter the average value in the passband fell from 239 to 39 (scale 0 to 255).

## 0.3.8 (calmer tabs while tuning)

- Turning the tuning knob no longer opens a tab per step. A new conversation starts only when the frequency has stayed put for 3 seconds. Coming back close to the old frequency (within 500 Hz) cancels it. A frequency typed in the app starts the conversation at once.

## 0.3.7 (updates from within the app)

- The app looks for a newer version on GitHub at start and with the button Check for updates (Settings). A bar shows the new version; Update now downloads, installs and restarts. The release notes are shown in the settings. No personal data is sent.
- Updates are signed with our own key; the app only installs packages that match the public key in `tauri.conf.json`.
- New workflow `release.yml`: pushing a tag `vX.Y.Z` builds the installer, signs it, makes a GitHub Release and uploads `latest.json`.
- Windows installer installs for the current user, so updating needs no administrator rights.
- Step by step guide to publish a version: `docs/nieuwe_versie_publiceren.md`.

## 0.3.6 (decoder tuned on real recordings)

- Automatic pitch: when the current pitch has been quiet for 1.5 s, the decoder moves to the strongest keyed tone between 300 and 1200 Hz (new `ToneSeeker`, local noise reference so the radio's filter shape does not fool it). The last 3 s of audio are decoded again at the new pitch, so the start of the signal is not lost. A click on the waterfall holds the pitch for 10 s. Can be switched off in the settings.
- The waterfall line follows the pitch the decoder actually uses.
- Signal threshold default raised from 2 to 3.5: on a real empty band the decoder made 122 characters of noise per minute, now 11.
- Own transmission: when the app starts sending, the decoder first hands out what it still held of the other station and then starts afresh. Our own sidetone is dropped until the radio is back on receive, also when the radio sends slower than estimated. The paddle gets the same clean start and end.

## 0.3.5 (conversations in tabs, waterfall zoom, signal threshold)

- Every conversation has its own tab with its own QSO details. A frequency change of more than 500 Hz, a new callsign or the button New QSO starts a new one. The earlier ones stay readable, can be deleted, and are saved on this computer (at most 40).
- Sending is only possible in the live tab. Decoded text always goes to the live tab, also while you look back.
- Waterfall: zoom 1x, 2x and 4x around the pitch, frequency scale, a calmer colour curve so noise stays dark.
- Choosing another audio input restarts the audio at once.
- New signal indicator next to the certainty meter and a signal threshold (squelch, 1 to 8) in the audio settings. A higher value makes the decoder trust only clearer signals.

## 0.3.4 (icon and Windows build)

- New application icon (set in `src-tauri/icons`, favicon for the UI).
- GitHub Actions workflow `windows-build` produces a Windows installer as a downloadable artifact.
- Dutch install guide in `docs/installatie_windows.md`.

## 0.3.3 (audio input and live decoding)

- New crate `cwld-audio`: sound card input (cpal), lock-free ring buffer, resampler to 8 kHz, waterfall (own FFT), WAV recording, audio thread with events.
- The waterfall shows the real spectrum; a click on a signal sets the decoder pitch.
- Decoded text appears live in the conversation, unsure characters faded, the uncertain tail in italics.
- Own typed text is not shown twice; other text heard while transmitting is the paddle.
- Record button in the status bar (Documents\CWLD, date, time and frequency in the file name).
- Settings: audio input, pitch, auto start, recording folder, input level meter, decode a WAV file.
- The certainty meter shows real values.

## 0.3.2 (decoder: interference and weak signals)

- Noise blanker against short strong clicks, automatic notch against steady carriers near the tone.
- The key-down level follows fading.
- Word model: Q-codes, abbreviations, callsign shapes and numbers choose between uncertain readings; weak lone E and T are dropped.
- Long pauses are searched at no extra cost; real time with margin.
- Generator: impulse noise and steady carriers. Bench: `--plain` for a before/after comparison.
- Measured results in `docs/meetresultaten_decoder.md`.

## 0.3.1 (certainty meter)

- New meter next to the S-meter: how sure the decoder is about the last 30 characters, with the signal to noise ratio. It is an estimate. In practice mode the real accuracy is shown, because the keyed text is known. For now it is fed by the demo, the real decoder is connected in the next step.

## 0.3.0 (decoder, work in progress)

- New crate `cwld-dsp`: synthetic CW generator (noise, fading, drift, sloppy keying, second station), tone front-end with AFC and presence test, WAV reader and writer.
- New crate `cwld-decoder`: probabilistic decoder (semi-Markov Viterbi, speed search, character confidence) and the test bench `cwld_bench`. First results in `docs/meetresultaten_decoder.md`.
- Not yet connected to audio input or the interface.

# Changelog

## 0.2.1 (SWR protection)

- SWR is read from the radio while it transmits (Icom, every 100 ms). Two readings in a row above the limit stop the text at once and show a red alert. Sending stays blocked until the operator confirms that the antenna was checked.
- Settings: protection on or off (default on), limit (default 1.5, kept between 1.1 and 3.0), QRX as the very last transmission after the stop (default on, can be switched off).
- The QMX gives SWR with the CAT command `SW;` (answer `SWnnn;`, hundredths) according to the QRP Labs CAT manual as quoted by the operator. This is read every 100 ms while transmitting and is not yet verified on a real radio. A bare `SW;` answer (receive) is no reading.

## 0.2.0 (phase 2, radio and sending)

- Icom IC-7300 MK II driver over CI-V: live frequency, mode, speed, S-meter and TX status, text sent in blocks of at most 30 characters, stop with `FF`.
- QRP Labs QMX driver over CAT: live status, text sent word by word into the 80 character `KY` buffer, paddle text from the radio decoder (`TB`) shown in the conversation.
- Keyer: capitals, unsupported characters left out and reported, prosigns per radio, blocks cut on word boundaries, one minute limit per message.
- Queue of waiting blocks visible next to the Send button. Esc empties it and stops the radio.
- Radio settings screen with port search, CI-V address, advanced options and plain-language help.
- A radio that is not connected shows nothing (no fake data). Port errors and warnings are explained in the interface.
- Everything tested against a fake serial port with the exact bytes. **Not yet tested on real radios**: see `docs/testplan_radio.md`.

## 0.1.1

- Conversation shows what you send with the paddle (decoded, dashed bubble "You (paddle)") next to typed text.
- The decoded callsign fills `{CALL}` automatically, plus `{NAME}` and `{QTH}` when the other station sent them. A callsign you type or click stays fixed until you ask to follow the decoder again.
- Variables: `{NAME}` and `{QTH}` are the other station, `{MYNAME}` and `{MYQTH}` are yours.

## 0.1.0 (phase 1, base)

- Workspace with crates `cwld-types`, `cwld-radio`, `cwld-core` and the Tauri 2 shell.
- Transmit guard: three modes (LIVE, LISTEN, PRACTICE). Only LIVE can reach the radio. CWLD never starts in LIVE.
- Radio simulator, so everything can be tried without a radio.
- Svelte 5 interface (direction A: conversation, narrow waterfall strip, chips, QSO panel, optional signals panel).
- English and Dutch, switchable without restart. Dark and light theme.
- Settings in one JSON file, written atomically. A corrupt file is kept as `.bad`.
- Demo data only. No real radio, decoder or audio yet.
