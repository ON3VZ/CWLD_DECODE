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
