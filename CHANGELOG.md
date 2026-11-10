## 0.14.0 (Koch method: choose your characters, copy by ear)

- New card "Characters to learn (Koch)" in the practice panel: all 40 characters in Koch order (K M R S U A P T L O W I N J E F 0 Y V G 5 / Q 9 Z H 3 8 B ? 4 2 7 C 1 D 6 X). A slider takes the first N characters, and you can switch single characters on or off. At least two stay. The choice is saved. Default: all characters, so nothing changes until you narrow it down.
- The paddle exercises now use only the chosen characters: groups, words and abbreviations, callsigns and QSO phrases. A word, callsign or phrase is only asked when it is made entirely of chosen characters; with too few of them you get character groups and a short note says so. The newest character and the ones that went wrong before come back more often.
- New listening exercise (copying): CWLD plays groups of 5 characters or words from your characters on this computer, never through the radio. Characters run at their own speed (default 20 WPM) with longer gaps (Farnsworth, default 12 WPM overall), pitch adjustable. You type what you hear (Enter checks, Esc stops, spaces do not matter); wrong characters are marked and counted per character.
- The 90 percent rule: after three listening rounds in a row with the same characters and together at least 90 percent right (at least 30 characters), CWLD proposes the next Koch character. It never adds one by itself.
- Internal: `schedule`, `durationSeconds` and `playSidetone` take an optional overall speed for Farnsworth timing; settings gain `practice.koch` (old settings files get all characters).

## 0.13.2 (paddle practice recognises the sidetone)

- Fixed: paddle practice heard the sidetone but showed nothing. A radio sidetone with overtones or slight clipping was taken for speech by the speech check from 0.12.0, which then held the decoder. The speech check is now off while the paddle trainer listens.
- Paddle practice reads your text straight from the key times, with your own speed measured over the whole attempt. The live decoder often misread the first letter while it was still finding your speed.
- The key times come from a new sidetone detector on the signal envelope (floor and peak with hysteresis): exact with a clean sidetone and with band noise up to 25 dB SNR in the tests, and band noise alone never counts as keying. Before, noise gave up to six times too many key-downs and audio heard again after a pitch move was counted twice.

## 0.13.1 (paddle practice works on the IC-7300 MK II, QRZ.com fixes)

- Practice mode no longer turns the radio's AF volume down. On the IC-7300 MK II the AF volume also sets how loud the sidetone is on the USB audio, so at 0 the paddle trainer heard nothing (found by ON3VZ). CWLD now closes the squelch instead (CI-V 14 03, still to confirm on the MK II) and switches break-in off; both are put back when you leave practice mode.
- The practice panel shows what the radio reports: volume, squelch and break-in, or "not reported". The paddle card warns when the AF volume is 0.
- Handbook section 24 now explains the radio setting that puts the sidetone on the USB audio: MENU, SET, Connectors, USB AF/IF Output, AF Beep/Speech... Output ON.
- Every practice exercise (simulator and paddle) starts in its own "Practice" tab, so practice text never mixes with a real QSO.
- Fixed: a callsign typed in the QSO panel was never looked up on QRZ.com; only clicked or decoded calls were. The grey QRZ.com line now only shows for the call in the panel.
- Fixed: name and QTH that CWLD filled in for the previous station are replaced when the callsign changes. What you typed yourself is never replaced.

## 0.13.0 (QRZ.com lookup)

- New in Settings, Logging link: QRZ.com lookup. With a QRZ.com account CWLD fetches the first name and the town of the station in the QSO panel and fills the Name and QTH fields. Only empty fields are filled; what you typed is never replaced. A small line under the fields shows what QRZ.com said (name, town and country, locator). Off by default.
- The password is kept encrypted by the operating system (Windows Credential Manager, Secret Service on Linux), never in the settings file, the log or the interface. "Save and test" logs in once and tells you whether the account has the XML subscription. "Remove stored password" deletes it.
- Only the callsign you are working is sent, over HTTPS, after you stopped typing for a moment. Answers are cached for a day (unknown calls for ten minutes). A failed lookup shows one quiet warning; logging and sending go on as before. Calls like ON3VZ/P fall back to the base call.
- Fixed: on a narrow window the paddle practice card no longer overlaps the panels below it.

## 0.12.0 (paddle practice, quiet radio, no text from SSB)

- Paddle practice: in PRACTICE, key with the paddle on the radio while nothing is transmitted. CWLD reads your sidetone from the USB audio and shows what it heard. Exercises: character groups, your weak characters, words and abbreviations, callsigns and QSO phrases. You get a score, the share of right characters, your speed, and your spacing (gaps inside characters, between characters and between words; with a straight key also the dash length and how even your elements are), with tips. CWLD remembers per character how often it went wrong, and the weak characters come back more often. The last results are shown with UTC time.
- Radio quiet while practising (on by default): entering PRACTICE turns the speaker of the IC-7300 MK II down (AF level 0) and switches break-in off, so you only hear CWLD and the paddle only makes the sidetone. Both are put back when you leave PRACTICE, change radio or close CWLD. A volume you turn up yourself during practice is left alone. To confirm on the MK II: the CI-V commands for AF level and break-in (test A40), and that the sidetone reaches the USB audio (test I10 and A41).
- No more text from SSB: when the radio is on USB, LSB, AM, FM or RTTY, the decoder shows nothing and says so above the conversation. On CW, a new check sees speech (an SSB station on the frequency) and the decoder waits until it stops. On the test bench, synthetic speech gave 36 to 54 characters a minute before and 0 to 5 now at good signal levels; the CW results stayed exactly the same.
- Internal: the radio reads of AF level and break-in are answered with data or refused; a refusal is never taken for a refused CW block.

## 0.11.0 (logging link: Log4OM and N1MM format)

- New section in Settings: Logging link. Every real QSO you log can be sent over UDP to Log4OM as one ADIF record per message (service type ADIF_MESSAGE, default port 2333). Off by default. The handbook (section 23) has the click by click steps for Log4OM.
- Optional second link in the format of the N1MM Logger+ broadcasts (contactinfo), with one or more destinations as address:port (default 127.0.0.1:12060). N1MM itself cannot receive QSOs over UDP, so this is meant for programs that listen to such broadcasts, not for logging into N1MM.
- Test QSOs are never sent. A failing send never blocks logging. The button "Send last QSO again" repeats your newest real QSO as a connection test; no fake QSO is ever made.
- To confirm on a real Log4OM: that it takes one raw ADIF record per UDP message (test A39).

## 0.10.4 (emergency stop with QRX, error signal)

- Esc and the Stop button work as before: sending stops at once and the queue is cleared. If something was being sent, a message says that a second Esc within two seconds also sends QRX QRX.
- Emergency stop: press Esc twice quickly (or click the new Stop + QRX button). The keyer and the queue are stopped and QRX QRX is sent, so the other station waits while you sort out your mistake. A second Esc only counts when the first one stopped a transmission, so an Esc on a quiet radio never sends anything.
- New Error button: stops what is being sent and sends the error signal (eight dots), so you can carry on with the correct word. The text in your send box is kept.
- Nothing is sent in LISTEN mode. In PRACTICE the QRX and the error signal are only simulated.
- To confirm on the radio: that the IC-7300 MK II accepts new text right after a stop (the QRX), and the prosign HH as `^HH` through CI-V.

## 0.10.3 (everything in UTC)

- All times CWLD shows are UTC now. The time above each message in the conversation and the start time of a tab used the clock of your computer, which is local time. They now use UTC, like the logbook, the ADIF file and the export window already did.
- A UTC clock with seconds in the status bar, next to the radio status.
- Checked: a QSO is stored as a moment in time (seconds since 1970), and the ADIF file writes QSO_DATE and TIME_ON from that in UTC, whatever the time zone or summer time setting of the computer.

## 0.10.2 (more room on the screen)

- QSO panel: the line "No station selected. Click a callsign in the conversation." is gone from the screen. It is now a tooltip on the callsign field while that field is empty. RST and RST rcvd sit next to each other on one line.
- Audio panel: the buttons Weak, Medium, Strong, Narrow, Wide, Strongest and AFC and the gain slider are behind a small Tuning switch (a triangle). They are hidden by default and CWLD remembers your choice on this computer. The waterfall and the peak meter stay visible, and the keys F6 and F7 (Ctrl+F6 and Ctrl+F7 in contest mode) keep working when the buttons are hidden.
- Activation: the activation panel on the right is replaced by a green button at the right end of the tab bar, next to New QSO and Close all. Without an activation it says Activation and opens the form. During an activation it shows the reference and the number of QSOs (for example ONFF-0599 with a counter), with a red dot when the activation crosses midnight UTC. Click it for the grid in large letters, the counts, the target and the End activation button.

## 0.10.1 (installer: the real cause of "Failed to kill CWLD")

- The installer of Tauri does not look for a running cwld.exe. It asks the Windows Restart Manager which programs have the file cwld.exe open and tries to shut those down. When that is a program CWLD cannot shut down (an antivirus scan, Windows Search, Explorer, a backup tool), the installer stopped with "Failed to kill CWLD", even though CWLD itself was closed. The install window then showed "CWLD has stopped" followed by "Failed to kill CWLD".
- CWLD now replaces that check by its own. The installer only stops when cwld.exe is really still running, with a clear message. When another program only has the file open, the install window names that program (process id and path), moves the old cwld.exe aside and carries on. The old file is removed later.
- This works from the first installation of this version on, because the new installer does the check. Installing 0.10.1 over an older version needs no manual steps, except when you choose in the installer to uninstall the old version first: that runs the old uninstaller.

## 0.10.0 (guided first start)

- A guided start in four steps opens on the first run, and again from Settings, Updates and about, Guided start. You can skip it at any time, and it never transmits.
- Step 1, find the radio: choose IC-7300 MK II, QMX or the simulator. CWLD looks at the serial ports and chooses one by itself when only one looks like your radio (for the IC-7300 MK II the CI-V port). When no port is found it lists three checks and how to open Device Manager. A test button connects and shows the frequency.
- Step 2, check the radio: explains CI-V Transceive with the menu path on the radio, and shows the live frequency. Turn the knob: when the frequency follows, it is confirmed. CWLD cannot read that setting from the radio, so this test is the check. For the QMX it points to the sidetone.
- Step 3, choose the sound: suggests the input that looks like the radio, starts listening and shows the level bar with the hint. Short Windows help for the microphone privacy switches and audio enhancements, and for the QMX the sidetone menu (Audio, Sidetone volume and Sidetone abs/rel).
- Step 4, your station: callsigns, name, QTH and locator, with the same checks as in Settings. A silent test shows the CQ that CWLD would send and how long it takes, without sending anything. At the end CWLD switches on connect at start and listen at start for what worked.
- The text of the wizard is in English and Dutch.

## 0.9.0 (practice against simulated stations)

- PRACTICE is now a simulator. Nothing is ever transmitted. The decoder listens to a practice band instead of the radio, and the stations are heard on the computer (headphones or speakers). Turn the AF knob of the radio down while you practise.
- Three exercises in the new Practice panel: a station calls CQ and you answer it, you call CQ and a station answers, and a contest in Run with the chosen contest profile (callers, pile-ups, F1 to F8 and Enter as in a real contest).
- The stations react to what you send: they answer your call, repeat on AGN or ?, give the number again on NR?, repeat their call when you send part of it with a question mark, correct a wrong call, and call again or leave when you keep them waiting.
- Settings per exercise: speed range, signal strength (weak, medium, strong), band noise, fading (QSB), most callers at once, length of the contest exercise, decoder help on or off (off hides the decoded text, so you copy by ear) and the volume.
- QSOs go to the test log. At the end a score shows the correct QSOs, wrong calls (with the real call), wrong exchanges, QSOs you completed but did not log, and the rate.
- A link to the morse learning app (on3vz.github.io/CWLD) in the Practice panel.
- The simulated callers come from a rough model of prefixes and zones, and the UBA sections they send are an illustrative set, not the official list.

## 0.8.0 (station profiles, activations, export variants)

- Station profiles Home and Portable, chosen in the status bar. The profile is never switched silently, and it is locked while an activation runs. The suffix (for example /P) is added to your base call on air. Each profile has grid, power, radio, antenna and a comment, edited in Settings, Station.
- Activations (WWFF/ONFF, also POTA and SOTA): a panel with reference, grid in large letters, the QSO counter and a midnight UTC warning. The reference is checked for its shape (ONFF-0599). A portable activation needs a grid. The grid, power and reference are written into every QSO of the activation. The target of 44 QSOs is shown as "to verify" until it is confirmed with the ONFF coordinator.
- Park to park: a field for their reference in the QSO panel while an activation runs.
- ADIF: station call, grid, power, MY_SIG/MY_SIG_INFO (WWFF), MY_POTA_REF, MY_SOTA_REF, SIG_INFO of the other station, and a WWFF file name (for example `ON3VZ-P @ ONFF-0599 20261103.adi`).
- Export variants: current activation, all, only QSOs not exported yet, contest and test log. Every exported QSO remembers that it was exported.
- A check window before export. Errors block the export (missing grid when portable, bad reference, QSO outside the activation time, missing report). Warnings do not (duplicates, several grids or station calls). The grid that goes into the file is shown in large letters.
- Band scope height is a setting: small (90 px), medium (165 px), large (230 px). Default: small in contest mode, medium otherwise.

## 0.7.0 (contest mode, logbook, ADIF export)

- One logbook for the whole app, also for normal QSOs. It is a SQLite file (`logbook.db`) next to the settings, with a backup copy made at every start. A deleted QSO is only marked, so nothing is lost by accident.
- Contest entry line (contest mode): callsign, report received (5NN by default), serial number received, the extra field of the profile (for example the UBA section, only for Belgian stations), your own number and a Log button. Space moves to the next field, Enter does the next logical thing.
- Function keys like N1MM Logger+, shown as buttons under the entry line with the key in small grey letters. F1 CQ (S&P: your call), F2 exchange, F3 TU and log, F4 your call, F5 his call, F6 QSO B4, F7 NR?, F8 AGN. Enter sends the right message (Run: CQ, exchange, AGN?, then TU and log). Ctrl+R switches between Run and S&P, Ctrl+W clears the line. In contest mode the decoder keys become Ctrl+F6 (Strongest) and Ctrl+F7 (AFC).
- The serial number goes up when you log, not when you send it, so repeating it does not skip a number. If you leave a station you sent a number to without logging, a warning names the number and the call.
- Dupes: the same call on the same band in the same contest shows a red DUPE (with time and band). Worked on another band shows in grey. Logging is still possible.
- Info line: band, number of QSOs, rate per hour over 10 minutes and the count of the last 60 minutes.
- Test mode (orange): QSOs go to a separate test log with its own number from 001, they never count for dupes and never reach the export. One button clears the test log. The real number stays as it was.
- New contest: switching contest mode on with a stored number asks Continue or Start again. Start again begins a new run at 001, and earlier QSOs no longer count as dupes.
- Click a word in decoded text to put it in the callsign, report, number, extra field, name or QTH. After you sent your exchange, the decoder proposes report, number and section from the reply in grey; Tab takes them over.
- Small log panel under the audio waterfall with the last 5 QSOs. Edit a QSO or delete it (asks first). Completing the extra field uses the values you logged before. The official UBA section list is not built in yet.
- Export (ADIF) with from/to (UTC), band and scope (real log, running contest, test log). The file is written in your CWLD folder (Documents). CONTEST_ID is an editable field in the export window: the suggested values still have to be checked against the official ADIF list.
- Normal QSOs: a Log button in the QSO panel, with a field for the report received.
- The band scope is lower (165 instead of 210 pixels, 90 in contest mode). In contest mode the screen is compact so the conversation stays readable on a laptop (1366x768): the normal chips, the pile-up row and the list of codes are hidden, the Contest button, Run/S&P, Test and the profile share one row, the speed slider sits next to the send box, and the rules note closes by itself after 12 seconds.
- After a click in the audio waterfall the AFC waits 4 seconds before it follows the signal, so it does not pull the tone away from where you clicked. (To be confirmed on a real signal.)
- Not yet: Cabrillo, station profiles and WWFF/ONFF activations (next round), the UBA section list, the messages behind F1 to F8 are fixed for now.

## 0.6.0 (contest profiles)

- Contest profiles. In contest mode a list on the main screen chooses the profile. A profile gives the CQ call buttons (for example TEST UBA) and the exchange you send, with a short note about the rules when you choose it. A profile can have more than one exchange (for example from Belgium and from abroad), chosen with a second list.
- Built in: UBA DX Contest, UBA ON Contest, UBA Spring Contest, UBA Low Band Winter Contest, CQ WW DX, CQ WPX, ARRL International DX, IARU HF Championship and WAE DX (all CW), and a blank template. They cannot be changed. Copy one to make your own version, or start a new profile. The rules were checked on 7 November 2026. The Spring and Winter rules come from older pages and must be checked before the contest.
- Own variables (Settings, Contest): `SEC` (WLD), `ZONE` (14), `ITU` (27) and `PWR` (20) are set at the start and can be changed. Add your own, for example `AGE`. They work as `{SEC}` in every chip and in the sending box. Names that CWLD uses itself cannot be taken.
- The editor shows how each exchange will look. Cut numbers for the serial number are an option per profile (019 becomes T1N), off by default.
- The extra field that you receive (for example the section) is set up in the profile. It is used when the logbook comes.
- Backup: export one profile, or all your profiles and variables, to a file, and import it again. The built-in profiles are not exported.
- The serial number is shown on the Contest button only when the chosen exchange uses it.
- Not yet: the logbook, counting up after logging, dupe warnings, the section list and QTC messages for WAE.

## 0.5.1 (installer fix, simulator made visible)

- The radio type "Simulator" is the default until you choose a radio. It always reported "Connected", which looked like a real radio. Now the status bar shows an orange "Simulator, no radio", and the real app shows a banner "No radio chosen yet" with a button that opens the settings.
- Installer: fixed "Failed to kill CWLD". After ending cwld.exe the installer now waits up to about 15 seconds until it is really gone (it holds the USB serial port and the sound card for a moment) and shows what it is doing in the install window. If CWLD does not stop, the window says how to end it in Task Manager and shows the process id and file path of what is still running.

## 0.5.0 (settings in sections, pile-up keys, tabs, decoder controls)

- Settings now has a menu on the left with sections: Station, Radio and SWR, Audio, Decoder, Sending, Display and language, Contest, Updates and about. No more one long list.
- The window starts maximised.
- The connection status follows the radio. When the radio is switched off or the cable is out, the status shows "Looking for radio" and CWLD tries to open the port again every 5 seconds (when "Connect when CWLD starts" is on). When the radio is back it connects by itself. (To be confirmed on your radio: the old driver already noticed silence after 3 seconds, so if the status still stays on connected, tell me what you did.)
- New standard report setting (Settings, Sending): 5NN (default) or 599. It fills the report in new conversations and in the chips.
- Pile-up keys: F1 your call, F2 your call twice, F3 "R" + report + "TU", F4 "AGN?". They also show as buttons under the chips. The Q-code panel moved from F1 to Ctrl+Q.
- Codes and abbreviations: words with a known meaning in the conversation have a dotted underline and a tooltip. Under the conversation a list "Codes in this conversation" shows everything that was received or sent, with its meaning. In the Q panel you can add your own abbreviations (tab Own).
- Tabs: a click on an earlier tab makes it the live conversation again and tunes the radio to its frequency (not while transmitting and not without a radio). Conversations without new text close by themselves after 15 minutes (Settings, Display, 0 switches it off). A conversation with sent text that is not logged never closes by itself. New button "Close all" keeps one empty conversation and asks first when sent text is not logged.
- Contest mode (basis): a Contest button under the chips, and a setting. In contest mode the variable {NR} is filled with the serial number (three digits) and an extra chip "Exchange" appears (`{CALL} {RST} {NR}`). Switching it on again with a stored number asks: continue or start again at 1. Counting up after each logged QSO comes with the logbook.
- Decoder controls, taken from what MRP40 offers: peak meter (5 to 90 percent is good, red above 90), input gain -20 to +20 dB, presets Weak/Medium/Strong (three positions of the squelch slider, chosen by feel and not measured), filter Narrow/Wide (double bandwidth), AFC switch (F7), "Strongest" once (F6), slowest and fastest speed (5 to 60 WPM, default 5 to 50) and a fixed speed. Red lines in the audio waterfall show the filter width.
- Measured: not yet. The benchmark against MRP40 is a separate step. Note from the filter comparison: the CWLD filter has much gentler flanks than the published MRP40 filter (see docs/meetresultaten_decoder.md when the benchmark is done).
- Not built yet: the "smart 4 second AFC" after a click when AFC is off, the logbook, dupe warnings, export and the sequence number counting up.

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
