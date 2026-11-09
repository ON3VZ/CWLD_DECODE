#!/usr/bin/env python3
"""Static site generator for the CWLD website (English at the root, Dutch under /nl/).
Run:  python3 site/build.py     Output: site/dist
No dependencies. The changelog page is generated from ../CHANGELOG.md."""
import json, os, re, shutil, struct
from html import escape
import svgs

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "src")
OUT = os.path.join(HERE, "dist")
REPO = "ON3VZ/CWLD_DECODE"
GH = f"https://github.com/{REPO}"
REL = f"{GH}/releases/latest"
OWN = "https://on3vz.github.io/"
TRAINER = "https://on3vz.github.io/CWLD/"
SITE = "https://on3vz.github.io/CWLD_DECODE/"

PAGES = ["index", "install", "help", "changelog"]
NAV = {
    "en": {"index": "Home", "install": "Install & connect", "help": "Help", "changelog": "Changelog"},
    "nl": {"index": "Home", "install": "Installeren & verbinden", "help": "Help", "changelog": "Wijzigingen"},
}

LOGO = ('<svg viewBox="0 0 32 32" aria-hidden="true"><rect width="32" height="32" rx="8" fill="#0f1822" stroke="#1f5f45"/>'
        '<circle cx="8" cy="16" r="2.6" fill="#3dffa8"/><rect x="13" y="13.4" width="11" height="5.2" rx="2.6" fill="#ffb020"/></svg>')


def png_size(path):
    with open(path, "rb") as f:
        d = f.read(24)
    return struct.unpack(">II", d[16:24])


def img(L, name, alt, cls="shot"):
    p = os.path.join(SRC, "img", f"{L}-{name}.png")
    w, h = png_size(p)
    return f'<img class="{cls}" src="{{R}}img/{L}-{name}.png" width="{w // 2}" height="{h // 2}" alt="{escape(alt)}" loading="lazy">'


def md_inline(s):
    s = escape(s, quote=False)
    s = re.sub(r"`([^`]+)`", r"<code>\1</code>", s)
    s = re.sub(r"\*\*([^*]+)\*\*", r"<b>\1</b>", s)
    return s


def changelog_html(L):
    md = open(os.path.join(HERE, "..", "CHANGELOG.md"), encoding="utf-8").read().splitlines()
    out, inl = [], False
    for ln in md:
        if ln.startswith("## "):
            if inl:
                out.append("</ul>"); inl = False
            out.append(f"<h2>{md_inline(ln[3:])}</h2>")
        elif ln.startswith("- "):
            if not inl:
                out.append("<ul>"); inl = True
            out.append(f"<li>{md_inline(ln[2:])}</li>")
        elif ln.strip() == "" or ln.startswith("# "):
            continue
        else:
            if inl:
                out.append("</ul>"); inl = False
            out.append(f"<p>{md_inline(ln)}</p>")
    if inl:
        out.append("</ul>")
    return "\n".join(out)


# ------------------------------------------------------------------ text
T = {
 "en": {
  "tag": "A CW station app for the IC-7300 MK II and the QMX. Open source.",
  "h1": "Read CW the way you hear it.",
  "lead": "CWLD decodes Morse with a probabilistic decoder that shows how sure it is, sends with your keyboard or with one click, keeps your logbook and practises with simulated stations. Free, open source, made by a radio amateur.",
  "dl": "Download for Windows", "guide": "Install & connect guide",
  "fine": ["Free and open source (GPL-3.0)", "No account, no cloud", "English and Dutch"],
  "feat_k": "What it does", "feat_h": "A station in one window",
  "feat_s": "Everything for a CW QSO, a contest or a park activation, with the radio still doing what it does best.",
  "feats": [
   ("RX", "Decoder that tells you how sure it is", "Instead of guessing silently, CWLD keeps several readings and shows doubt. Weak or messy signals are not hidden behind confident wrong letters."),
   ("TX", "Send from the keyboard", "Type, or click a ready-made answer. Esc stops at once. Your paddle on the radio keeps working at all times."),
   ("QSO", "A conversation, not a text dump", "Each station is a conversation. Click a callsign to put it in the QSO panel, click a chip to answer."),
   ("LOG", "Logbook and ADIF", "Contest entry line with F-keys like N1MM, dupes, rate, and an ADIF export with checks before it leaves."),
   ("/P", "Home and portable", "Station profiles and activations for WWFF, POTA and SOTA, with grid, power and reference in every QSO."),
   ("SIM", "Practice against simulated stations", "Answer a CQ, run a pile-up or play a contest. Nothing is ever transmitted."),
  ],
  "modes_k": "Three modes", "modes_h": "You always know whether you can transmit",
  "modes": [("live", "LIVE", "The only mode that transmits. Always shown in red."), ("listen", "LISTEN", "Receive and decode only. Safe for listening and testing."), ("practice", "PRACTICE", "Simulated stations through your headphones. Nothing leaves your computer.")],
  "two_k": "The screen", "two_h": "Made to be read at a glance",
  "two_p": "A narrow waterfall, big readable text, one colour for callsigns. Dark and light themes, English and Dutch without a restart, and a short explanation with every setting.",
  "two_l": ["Click a callsign to start a QSO", "Chips with {CALL}, {MYCALL}, {NAME}, {RST}, {QTH}", "Live frequency, band, mode and S-meter from the radio", "CWLD never transmits by itself"],
  "toy_k": "Try it", "toy_h": "How does your name sound?", "toy_p": "Type a word and play it as Morse. This runs in your browser and sends nothing anywhere.",
  "toy_label": "Speed", "toy_play": "Play", "toy_stop": "Stop", "toy_ph": "ON3VZ",
  "dl_k": "Download", "dl_h": "Get the latest version",
  "dl_ver": "Latest version", "dl_file": "File", "dl_size": "Size", "dl_date": "Released", "dl_sha": "SHA-256",
  "dl_nohash": "see the release page", "dl_btn": "Download the Windows installer", "dl_notes": "Release notes", "dl_fail": "Could not look up the latest version. Open the release page instead:", "dl_fail_a": "GitHub Releases",
  "dl_smart": "Windows may show a blue SmartScreen message because CWLD is not code signed yet. That is normal for a new open-source program. Click More info, then Run anyway. The install guide shows it with a picture.",
  "req_h": "System requirements",
  "reqs": [("Windows", "Windows 10 or 11, 64-bit"), ("Radio", "IC-7300 MK II or QRP Labs QMX, with a USB cable"), ("Linux", "Planned, not released yet"), ("macOS", "On the roadmap"), ("Privacy", "No account. Only the download button asks GitHub for the latest version.")],
  "promo_h": "Learning Morse first?", "promo_p": "The CWLD morse trainer is a free web app by the same author: learn the characters, build speed and practise copying.", "promo_a": "Open the morse trainer",
  "own_a": "More from ON3VZ",
  "f_about": "CWLD is a free CW station app. Made in Hoboken, Belgium by ON3VZ.",
  "f_proj": "Project", "f_guides": "Guides", "f_more": "Also by ON3VZ",
  "f_links": [("Source code", GH), ("Download", REL), ("Report a problem", GH + "/issues"), ("License: GPL-3.0", GH + "/blob/main/LICENSE")],
  "f_made": "73 de ON3VZ", "f_air": "Never transmits unattended",
  "demo": [{"who": "ON3VZ", "me": True, "text": "CQ CQ DE ON3VZ ON3VZ K"}, {"who": "DL1ABC", "me": False, "text": "ON3VZ DE DL1ABC DL1ABC KN"}, {"who": "ON3VZ", "me": True, "text": "DL1ABC DE ON3VZ GM OM UR 599 BK"}, {"who": "DL1ABC", "me": False, "text": "ON3VZ DE DL1ABC TNX FER CALL UR 599 73 SK"}],
  "demo_bar": "decoding at 18 WPM",
  # pages
  "i_title": "Install and connect", "i_desc": "Step by step, with pictures: install CWLD, connect your IC-7300 MK II or QMX and check that it works.",
  "i_toc": ["Install CWLD", "Connect the IC-7300 MK II", "Connect the QMX", "Check that it works", "If something is wrong"],
  "h_title": "Help", "h_desc": "Answers, keys, modes and where your data lives.",
  "c_title": "Changelog", "c_desc": "What changed in each version.", "c_note": "The changelog is written in English.",
 },
 "nl": {
  "tag": "Een CW-station-app voor de IC-7300 MK II en de QMX. Open source.",
  "h1": "Lees CW zoals je het hoort.",
  "lead": "CWLD decodeert Morse met een waarschijnlijkheidsdecoder die toont hoe zeker hij is, zendt met je toetsenbord of met één klik, houdt je logboek bij en laat je oefenen met gesimuleerde stations. Gratis, open source, gemaakt door een radioamateur.",
  "dl": "Download voor Windows", "guide": "Gids: installeren en verbinden",
  "fine": ["Gratis en open source (GPL-3.0)", "Geen account, geen cloud", "Nederlands en Engels"],
  "feat_k": "Wat het doet", "feat_h": "Een station in één venster",
  "feat_s": "Alles voor een CW-QSO, een contest of een park-activatie, terwijl de radio blijft doen waar hij goed in is.",
  "feats": [
   ("RX", "Een decoder die zegt hoe zeker hij is", "In plaats van stil te gokken houdt CWLD meerdere lezingen bij en toont twijfel. Zwakke of rommelige signalen verdwijnen niet achter zelfzekere foute letters."),
   ("TX", "Zenden met het toetsenbord", "Typ, of klik op een kant-en-klaar antwoord. Esc stopt meteen. Je paddle op de radio blijft altijd werken."),
   ("QSO", "Een gesprek, geen tekstmuur", "Elk station is een gesprek. Klik op een roepletter om ze in het QSO-paneel te zetten, klik op een chip om te antwoorden."),
   ("LOG", "Logboek en ADIF", "Contest-invoerlijn met F-toetsen zoals N1MM, dupes, tempo, en een ADIF-export met controles voor het bestand vertrekt."),
   ("/P", "Thuis en portable", "Stationsprofielen en activaties voor WWFF, POTA en SOTA, met grid, vermogen en referentie in elk QSO."),
   ("SIM", "Oefenen met gesimuleerde stations", "Beantwoord een CQ, draai een pile-up of speel een contest. Er wordt nooit iets uitgezonden."),
  ],
  "modes_k": "Drie modi", "modes_h": "Je weet altijd of je kunt zenden",
  "modes": [("live", "LIVE", "De enige modus die zendt. Altijd in het rood."), ("listen", "LISTEN", "Alleen ontvangen en decoderen. Veilig om te luisteren en te testen."), ("practice", "PRACTICE", "Gesimuleerde stations in je koptelefoon. Er verlaat niets je computer.")],
  "two_k": "Het scherm", "two_h": "Gemaakt om in één blik te lezen",
  "two_p": "Een smalle waterfall, grote leesbare tekst, één kleur voor roepletters. Donker en licht thema, Nederlands en Engels zonder herstart, en een korte uitleg bij elke instelling.",
  "two_l": ["Klik op een roepletter om een QSO te starten", "Chips met {CALL}, {MYCALL}, {NAME}, {RST}, {QTH}", "Live frequentie, band, modus en S-meter van de radio", "CWLD zendt nooit uit zichzelf"],
  "toy_k": "Probeer het", "toy_h": "Hoe klinkt jouw naam?", "toy_p": "Typ een woord en speel het af als Morse. Dit draait in je browser en stuurt nergens iets naartoe.",
  "toy_label": "Snelheid", "toy_play": "Afspelen", "toy_stop": "Stop", "toy_ph": "ON3VZ",
  "dl_k": "Download", "dl_h": "Haal de laatste versie",
  "dl_ver": "Laatste versie", "dl_file": "Bestand", "dl_size": "Grootte", "dl_date": "Uitgebracht", "dl_sha": "SHA-256",
  "dl_nohash": "zie de release-pagina", "dl_btn": "Download het Windows-installatieprogramma", "dl_notes": "Release-notities", "dl_fail": "De laatste versie kon niet worden opgezocht. Open dan de release-pagina:", "dl_fail_a": "GitHub Releases",
  "dl_smart": "Windows kan een blauwe SmartScreen-melding tonen omdat CWLD nog niet digitaal ondertekend is. Dat is normaal voor een nieuw open-sourceprogramma. Klik op Meer info en dan op Toch uitvoeren. De gids toont het met een afbeelding.",
  "req_h": "Systeemeisen",
  "reqs": [("Windows", "Windows 10 of 11, 64-bit"), ("Radio", "IC-7300 MK II of QRP Labs QMX, met een USB-kabel"), ("Linux", "Gepland, nog niet uitgebracht"), ("macOS", "Staat op de roadmap"), ("Privacy", "Geen account. Alleen de downloadknop vraagt GitHub om de laatste versie.")],
  "promo_h": "Eerst Morse leren?", "promo_p": "De CWLD-morsetrainer is een gratis webapp van dezelfde maker: leer de tekens, bouw snelheid op en oefen met opschrijven.", "promo_a": "Open de morsetrainer",
  "own_a": "Meer van ON3VZ",
  "f_about": "CWLD is een gratis CW-station-app. Gemaakt in Hoboken, België door ON3VZ.",
  "f_proj": "Project", "f_guides": "Gidsen", "f_more": "Ook van ON3VZ",
  "f_links": [("Broncode", GH), ("Download", REL), ("Probleem melden", GH + "/issues"), ("Licentie: GPL-3.0", GH + "/blob/main/LICENSE")],
  "f_made": "73 de ON3VZ", "f_air": "Zendt nooit onbewaakt",
  "demo": [{"who": "ON3VZ", "me": True, "text": "CQ CQ DE ON3VZ ON3VZ K"}, {"who": "DL1ABC", "me": False, "text": "ON3VZ DE DL1ABC DL1ABC KN"}, {"who": "ON3VZ", "me": True, "text": "DL1ABC DE ON3VZ GM OM UR 599 BK"}, {"who": "DL1ABC", "me": False, "text": "ON3VZ DE DL1ABC TNX FER CALL UR 599 73 SK"}],
  "demo_bar": "decoderen op 18 WPM",
  "i_title": "Installeren en verbinden", "i_desc": "Stap voor stap, met afbeeldingen: CWLD installeren, je IC-7300 MK II of QMX verbinden en controleren dat het werkt.",
  "i_toc": ["CWLD installeren", "De IC-7300 MK II verbinden", "De QMX verbinden", "Controleren of het werkt", "Als er iets mis is"],
  "h_title": "Help", "h_desc": "Antwoorden, toetsen, modi en waar je gegevens staan.",
  "c_title": "Wijzigingen", "c_desc": "Wat er in elke versie veranderde.", "c_note": "De lijst met wijzigingen is in het Engels.",
 },
}


def step(n, title, body):
    return f'<div class="step"><div class="n">{n}</div><div><h3>{title}</h3>{body}</div></div>'


def fig(inner, cap, narrow=False):
    return f'<figure{" class=\"narrow\"" if narrow else ""}>{inner}<figcaption>{cap}</figcaption></figure>'


# ------------------------------------------------------------------ install page
def install_body(L):
    en = L == "en"
    S = {k: v for k, v in [
        ("cable", svgs.cable(L)), ("menu", svgs.radio_menu(L)), ("dm", svgs.device_manager(L)),
        ("ss", svgs.smartscreen(L)), ("mic", svgs.mic(L)), ("qmx", svgs.qmx_menu(L)), ("flow", svgs.flow(L))]}
    t = T[L]
    toc = "".join(f'<a href="#s{i + 1}">{x}</a>' for i, x in enumerate(t["i_toc"]))
    h = []
    h.append(f'<div class="cols"><aside class="toc"><b>{"On this page" if en else "Op deze pagina"}</b>{toc}</aside><article>')
    # 1
    h.append(f'<h2 id="s1"><span class="letter">1</span>{t["i_toc"][0]}</h2>')
    h.append(fig(S["flow"], "How CWLD connects to your radio: one USB-C cable carries control and sound." if en else "Hoe CWLD met je radio verbindt: één USB-C-kabel voor bediening en geluid."))
    h.append(step(1, "Download the installer" if en else "Download het installatieprogramma",
        ('<p>Press the button below. The file ends in <code>-setup.exe</code>. Your browser may ask whether to keep it: choose Keep.</p>' if en else
         '<p>Druk op de knop hieronder. Het bestand eindigt op <code>-setup.exe</code>. Je browser kan vragen of je het wilt bewaren: kies Bewaren.</p>')
        + f'<p><a class="btn primary" data-dl href="{REL}">{t["dl_btn"]} <small data-version></small></a></p>'))
    h.append(step(2, "Start it and get past SmartScreen" if en else "Start het en kom voorbij SmartScreen",
        ('<p>Double-click the downloaded file. Windows may show a blue window, because CWLD is not code signed yet. Your computer is not in danger: the program is open source and every release is built on GitHub.</p>'
         '<ol><li>Click <b>More info</b> (1).</li><li>A button <b>Run anyway</b> appears (2). Click it.</li></ol>' if en else
         '<p>Dubbelklik op het gedownloade bestand. Windows kan een blauw venster tonen omdat CWLD nog niet digitaal ondertekend is. Je computer loopt geen gevaar: het programma is open source en elke versie wordt op GitHub gebouwd.</p>'
         '<ol><li>Klik op <b>Meer info</b> (1).</li><li>Er verschijnt een knop <b>Toch uitvoeren</b> (2). Klik erop.</li></ol>')
        + fig(S["ss"], "The SmartScreen message in two steps." if en else "De SmartScreen-melding in twee stappen.")
        + ('<div class="note"><b>Antivirus</b>If your antivirus blocks the installer, you can compare the SHA-256 checksum on the download section of the home page with the file you downloaded.</div>' if en else
           '<div class="note"><b>Antivirus</b>Blokkeert je antivirus het installatieprogramma, vergelijk dan de SHA-256-controlesom in het downloadgedeelte op de startpagina met het bestand dat je downloadde.</div>')))
    h.append(step(3, "Finish the installer" if en else "Rond de installatie af",
        '<p>Click Next, keep the suggested folder, click Install and then Finish. CWLD starts.</p>' if en else
        '<p>Klik op Volgende, behoud de voorgestelde map, klik op Installeren en dan op Voltooien. CWLD start.</p>'))
    # 2
    h.append(f'<h2 id="s2"><span class="letter">2</span>{t["i_toc"][1]}</h2>')
    h.append('<p>' + ('Do these steps in order. It takes about ten minutes the first time.' if en else 'Doe deze stappen in volgorde. De eerste keer duurt het ongeveer tien minuten.') + '</p>')
    h.append(step(1, "Plug in the USB-C cable" if en else "Steek de USB-C-kabel in",
        ('<p>Connect the USB-C port of the radio (1) to a USB port of your PC (2) with a cable that carries data. Switch the radio on.</p>' if en else
         '<p>Verbind de USB-C-poort van de radio (1) met een USB-poort van je pc (2) met een kabel die data doorgeeft. Zet de radio aan.</p>')
        + fig(S["cable"], "One cable is enough for control and sound." if en else "Eén kabel is genoeg voor bediening en geluid.")))
    h.append(step(2, "Turn CI-V Transceive on, in the radio" if en else "Zet CI-V Transceive aan, in de radio",
        ('<p>This makes the radio tell CWLD by itself when you turn the knob or change band, so the frequency on the screen follows live.</p>'
         '<ol><li>Press <b>MENU</b> (1).</li><li>Open <b>SET</b> (2).</li><li>Open <b>Connectors</b> (3).</li><li>Open <b>CI-V</b> (4).</li><li>Set <b>CI-V Transceive</b> to <b>ON</b> (5). Leave <b>CI-V Address</b> at its default, <code>B6</code>.</li></ol>' if en else
         '<p>Hierdoor meldt de radio zelf aan CWLD wanneer je aan de knop draait of van band wisselt, zodat de frequentie op het scherm live meebeweegt.</p>'
         '<ol><li>Druk op <b>MENU</b> (1).</li><li>Open <b>SET</b> (2).</li><li>Open <b>Connectors</b> (3).</li><li>Open <b>CI-V</b> (4).</li><li>Zet <b>CI-V Transceive</b> op <b>ON</b> (5). Laat <b>CI-V Address</b> op de standaardwaarde, <code>B6</code>.</li></ol>')
        + fig(S["menu"], "The menu path on the radio. Names can differ slightly between firmware versions." if en else "Het menupad op de radio. Namen kunnen licht verschillen tussen firmwareversies.")))
    h.append(step(3, "Find the COM port in Windows" if en else "Zoek de COM-poort in Windows",
        ('<ol><li>Right-click the <b>Start</b> button and choose <b>Device Manager</b>.</li><li>Click the small arrow next to <b>Ports (COM &amp; LPT)</b>.</li><li>You see two ports for the radio. Remember the number of <b>Serial Port A (CI-V)</b> (1), for example COM5.</li></ol>'
         '<div class="tip"><b>Tip</b>If CWLD gets no answer on port 1, try the other port (2). Your COM numbers will be different from the picture.</div>' if en else
         '<ol><li>Klik met de rechtermuisknop op de knop <b>Start</b> en kies <b>Apparaatbeheer</b>.</li><li>Klik op het pijltje naast <b>Poorten (COM en LPT)</b>.</li><li>Je ziet twee poorten voor de radio. Onthoud het nummer van <b>Serial Port A (CI-V)</b> (1), bijvoorbeeld COM5.</li></ol>'
         '<div class="tip"><b>Tip</b>Krijgt CWLD geen antwoord op poort 1, probeer dan de andere poort (2). Jouw COM-nummers zullen anders zijn dan op de afbeelding.</div>')
        + fig(S["dm"], "Device Manager with the two radio ports." if en else "Apparaatbeheer met de twee radiopoorten.")))
    h.append(step(4, "Fill in your station in CWLD" if en else "Vul je station in, in CWLD",
        ('<p>Click the <b>gear icon</b> at the top right of CWLD to open Settings, then the <b>Station</b> tab. Enter your own callsign. Grid and power are optional for now.</p>' if en else
         '<p>Klik op het <b>tandwiel</b> rechtsboven in CWLD om de instellingen te openen en dan op het tabblad <b>Station</b>. Vul je eigen roepletters in. Grid en vermogen zijn voorlopig optioneel.</p>')
        + fig(img(L, "settings-station", "CWLD settings, Station tab" if en else "CWLD-instellingen, tabblad Station"), "Settings, Station." if en else "Instellingen, Station.")))
    h.append(step(5, "Connect the radio" if en else "Verbind de radio",
        ('<p>Open the <b>Radio</b> tab.</p><ol><li>Choose the <b>Radio type</b> (1): IC-7300 MK II.</li><li>Choose the <b>Port</b> (2) from step 3.</li><li>Check that the <b>CI-V address</b> (3) is <code>B6</code>.</li><li>Click <b>Connect</b> (4).</li></ol>'
         '<p>Tick <b>Connect when CWLD starts</b> if you want this to happen by itself next time.</p>' if en else
         '<p>Open het tabblad <b>Radio</b>.</p><ol><li>Kies het <b>Radiotype</b> (1): IC-7300 MK II.</li><li>Kies de <b>Poort</b> (2) uit stap 3.</li><li>Controleer dat het <b>CI-V-adres</b> (3) <code>B6</code> is.</li><li>Klik op <b>Verbinden</b> (4).</li></ol>'
         '<p>Vink <b>Verbinden bij het starten van CWLD</b> aan als je wilt dat dit de volgende keer vanzelf gebeurt.</p>')
        + fig(img(L, "settings-radio", "CWLD settings, Radio tab with numbered steps" if en else "CWLD-instellingen, tabblad Radio met genummerde stappen"), "Settings, Radio." if en else "Instellingen, Radio.")))
    h.append(step(6, "Choose the audio" if en else "Kies het geluid",
        ('<p>Open the <b>Audio</b> tab. Choose <b>USB Audio CODEC</b> as audio input (1), then click <b>Restart with these settings</b> (2).</p>' if en else
         '<p>Open het tabblad <b>Audio</b>. Kies <b>USB Audio CODEC</b> als audio-ingang (1) en klik dan op <b>Herstart met deze instellingen</b> (2).</p>')
        + fig(img(L, "settings-audio", "CWLD settings, Audio tab with numbered steps" if en else "CWLD-instellingen, tabblad Audio met genummerde stappen"), "Settings, Audio." if en else "Instellingen, Audio.")))
    h.append(step(7, "Allow CWLD to use the microphone input" if en else "Sta toe dat CWLD de microfooningang gebruikt",
        ('<p>For Windows the radio sound counts as a microphone. If you see no waterfall, check this:</p>'
         '<ol><li>Click <b>Start</b>, type <b>microphone privacy</b> and press Enter.</li><li>Switch on the three toggles shown below.</li><li>Also open the sound settings of Windows and turn <b>audio enhancements</b> off for the USB Audio CODEC.</li></ol>' if en else
         '<p>Voor Windows telt het radiogeluid als microfoon. Zie je geen waterfall, controleer dan dit:</p>'
         '<ol><li>Klik op <b>Start</b>, typ <b>microfoon privacy</b> en druk op Enter.</li><li>Zet de drie schakelaars aan zoals hieronder.</li><li>Open ook de geluidsinstellingen van Windows en zet de <b>geluidsverbeteringen</b> uit voor de USB Audio CODEC.</li></ol>')
        + fig(S["mic"], "The three Windows privacy toggles." if en else "De drie privacyschakelaars van Windows.", True)))
    # 3
    h.append(f'<h2 id="s3"><span class="letter">3</span>{t["i_toc"][2]}</h2>')
    h.append('<p>' + ('The QMX also uses one USB-C cable. It shows up in Windows as a virtual COM port.' if en else 'Ook de QMX gebruikt één USB-C-kabel. Hij verschijnt in Windows als een virtuele COM-poort.') + '</p>')
    h.append(step(1, "Plug in and find the port" if en else "Steek in en zoek de poort",
        '<p>Connect the QMX and find its COM port in Device Manager as in step 3 above.</p>' if en else
        '<p>Verbind de QMX en zoek de COM-poort in Apparaatbeheer zoals in stap 3 hierboven.</p>'))
    h.append(step(2, "Make the sidetone loud enough" if en else "Maak de zijtoon luid genoeg",
        ('<p>CWLD listens to the sidetone of the QMX. In the menu of the QMX open <b>Audio</b>, set <b>Sidetone volume</b> high (1), for example 99, and <b>Sidetone abs/rel</b> to <b>Relative</b> (2).</p>' if en else
         '<p>CWLD luistert naar de zijtoon van de QMX. Open in het menu van de QMX <b>Audio</b>, zet <b>Sidetone volume</b> hoog (1), bijvoorbeeld 99, en <b>Sidetone abs/rel</b> op <b>Relative</b> (2).</p>')
        + fig(S["qmx"], "QMX menu: Audio." if en else "QMX-menu: Audio.")))
    h.append(step(3, "Connect in CWLD" if en else "Verbind in CWLD",
        '<p>Do the same as for the IC-7300 MK II: Settings, Radio, choose the QMX as radio type, the port, then Connect. Then choose the audio input of the QMX in the Audio tab and restart.</p>' if en else
        '<p>Doe hetzelfde als voor de IC-7300 MK II: Instellingen, Radio, kies de QMX als radiotype, de poort, en dan Verbinden. Kies daarna de audio-ingang van de QMX in het tabblad Audio en herstart.</p>'))
    h.append('<div class="note"><b>Windows 11</b>' + ('Windows 11 allows at most two such virtual ports. If the QMX does not appear, remove ports you no longer use in Device Manager.' if en else 'Windows 11 laat hoogstens twee van zulke virtuele poorten toe. Verschijnt de QMX niet, verwijder dan poorten die je niet meer gebruikt in Apparaatbeheer.') + '</div>')
    # 4
    h.append(f'<h2 id="s4"><span class="letter">4</span>{t["i_toc"][3]}</h2>')
    h.append('<div class="check-end"><h3>' + ('You are done when you see this' if en else 'Je bent klaar als je dit ziet') + '</h3><ul class="checks">'
        + ''.join(f'<li>{x}</li>' for x in (
            ["The status bar says the radio is connected.", "You turn the knob on the radio and the frequency on the screen follows at once.", "The waterfall moves and you see a signal when a CW station is on.", "In LISTEN mode the decoded text appears in the conversation."] if en else
            ["De statusbalk meldt dat de radio verbonden is.", "Je draait aan de knop van de radio en de frequentie op het scherm volgt meteen.", "De waterfall beweegt en je ziet een signaal als er een CW-station is.", "In LISTEN-modus verschijnt de gedecodeerde tekst in het gesprek."]))
        + '</ul></div>')
    h.append(fig(img(L, "main", "The CWLD main screen" if en else "Het hoofdscherm van CWLD"), "The main screen." if en else "Het hoofdscherm."))
    h.append('<div class="warn"><b>' + ('Before you transmit' if en else 'Voor je zendt') + '</b>' + (
        'Only the LIVE mode transmits. Start with low power into a dummy load or a free frequency, and remember that Esc stops sending at once. You are responsible for following the rules of your licence.' if en else
        'Alleen de LIVE-modus zendt. Begin met laag vermogen op een dummy load of een vrije frequentie, en onthoud dat Esc het zenden meteen stopt. Jij bent verantwoordelijk voor het volgen van de regels van je licentie.') + '</div>')
    # 5
    h.append(f'<h2 id="s5"><span class="letter">5</span>{t["i_toc"][4]}</h2>')
    faq = faq_items(L)[:4]
    h.append(''.join(f'<details><summary>{q}</summary><div class="a">{a}</div></details>' for q, a in faq))
    h.append('<p>' + ('More answers are on the <a href="{HELP}">Help page</a>.' if en else 'Meer antwoorden staan op de <a href="{HELP}">helppagina</a>.') + '</p>')
    h.append('</article></div>')
    return ''.join(h)


def faq_items(L):
    en = L == "en"
    return [
     ("CWLD does not find the radio" if en else "CWLD vindt de radio niet",
      '<p>Check that the radio is on and the cable carries data. Open Device Manager (right-click Start) and look under Ports (COM &amp; LPT). Choose the first port in CWLD, and if there is no answer, the second one. Check that the CI-V address is <code>B6</code>.</p>' if en else
      '<p>Controleer dat de radio aan staat en de kabel data doorgeeft. Open Apparaatbeheer (rechtsklik op Start) en kijk onder Poorten (COM en LPT). Kies in CWLD de eerste poort en, als er geen antwoord komt, de tweede. Controleer dat het CI-V-adres <code>B6</code> is.</p>'),
     ("The frequency on the screen does not follow the radio" if en else "De frequentie op het scherm volgt de radio niet",
      '<p>Switch <b>CI-V Transceive</b> on in the radio: MENU, SET, Connectors, CI-V. CWLD also asks the radio for the frequency now and then, but Transceive makes it instant.</p>' if en else
      '<p>Zet <b>CI-V Transceive</b> aan in de radio: MENU, SET, Connectors, CI-V. CWLD vraagt de radio ook af en toe om de frequentie, maar met Transceive gaat het meteen.</p>'),
     ("The waterfall is empty and nothing is decoded" if en else "De waterfall is leeg en er wordt niets gedecodeerd",
      '<p>In Settings, Audio, choose <b>USB Audio CODEC</b> and click Restart with these settings. Then check the three microphone privacy switches of Windows and turn audio enhancements off. Make sure the radio has a CW signal in its filter and that the AF level is not zero.</p>' if en else
      '<p>Kies in Instellingen, Audio, <b>USB Audio CODEC</b> en klik op Herstart met deze instellingen. Controleer dan de drie microfoon-privacyschakelaars van Windows en zet geluidsverbeteringen uit. Zorg dat er een CW-signaal in het filter van de radio staat en dat het AF-niveau niet nul is.</p>'),
     ("Windows or my antivirus blocks the installer" if en else "Windows of mijn antivirus blokkeert het installatieprogramma",
      '<p>CWLD is not code signed yet, so Windows is careful. Click More info, then Run anyway. If an antivirus removes the file, compare its SHA-256 checksum with the one on the download section and allow the file.</p>' if en else
      '<p>CWLD is nog niet digitaal ondertekend, dus Windows is voorzichtig. Klik op Meer info en dan op Toch uitvoeren. Verwijdert een antivirus het bestand, vergelijk dan de SHA-256-controlesom met die in het downloadgedeelte en sta het bestand toe.</p>'),
     ("Does CWLD transmit by itself?" if en else "Zendt CWLD uit zichzelf?",
      '<p>No. Only the LIVE mode transmits, and only when you type, press a key or click a chip. Esc stops at once. LISTEN and PRACTICE never transmit.</p>' if en else
      '<p>Nee. Alleen de LIVE-modus zendt, en alleen als je typt, een toets indrukt of op een chip klikt. Esc stopt meteen. LISTEN en PRACTICE zenden nooit.</p>'),
     ("Does my paddle still work?" if en else "Werkt mijn paddle nog?",
      '<p>Yes. The keyer of the radio works on its own and CWLD never blocks it.</p>' if en else
      '<p>Ja. De keyer van de radio werkt zelfstandig en CWLD blokkeert hem nooit.</p>'),
     ("Where is my data?" if en else "Waar staan mijn gegevens?",
      '<p>Everything stays on your computer. Settings and the logbook (<code>logbook.db</code>, with a backup copy at every start) are in <code>%APPDATA%\\io.github.on3vz.cwld</code>. Recordings go to <code>Documents\\CWLD</code>. Press Windows+R, paste the folder name and press Enter to open it.</p>' if en else
      '<p>Alles blijft op je computer. Instellingen en logboek (<code>logbook.db</code>, met een reservekopie bij elke start) staan in <code>%APPDATA%\\io.github.on3vz.cwld</code>. Opnames gaan naar <code>Documenten\\CWLD</code>. Druk op Windows+R, plak de mapnaam en druk op Enter om ze te openen.</p>'),
     ("How do I update?" if en else "Hoe update ik?",
      '<p>Download the newest installer from the home page and run it over the old version. Your settings and logbook stay.</p>' if en else
      '<p>Download het nieuwste installatieprogramma van de startpagina en voer het uit over de oude versie. Je instellingen en logboek blijven bewaard.</p>'),
     ("How do I get my QSOs into my logger?" if en else "Hoe krijg ik mijn QSO's in mijn logprogramma?",
      '<p>Use the export function of the logbook. It writes an ADIF file, after a check window that lists errors and warnings. Import that file in Log4OM or any other logger.</p>' if en else
      '<p>Gebruik de exportfunctie van het logboek. Die schrijft een ADIF-bestand, na een controlevenster met fouten en waarschuwingen. Importeer dat bestand in Log4OM of een ander logprogramma.</p>'),
    ]


def help_body(L):
    en = L == "en"
    t = T[L]
    toc = [("modes", "Modes" if en else "Modi"), ("keys", "Keys" if en else "Toetsen"), ("faq", "Questions" if en else "Vragen"), ("contest", "Contest" if en else "Contest"), ("portable", "Portable" if en else "Portable"), ("practice", "Practice" if en else "Oefenen")]
    h = ['<div class="cols"><aside class="toc"><b>' + ("On this page" if en else "Op deze pagina") + '</b>' + ''.join(f'<a href="#{i}">{n}</a>' for i, n in toc) + '</aside><article>']
    h.append(f'<h2 id="modes"><span class="letter">M</span>{toc[0][1]}</h2><div class="modes">'
             + ''.join(f'<div class="mode {c}"><h3>{n}</h3><p>{d}</p></div>' for c, n, d in t["modes"]) + '</div>')
    h.append(fig(img(L, "main-modes", "The three modes in the CWLD toolbar" if en else "De drie modi in de werkbalk van CWLD"), "The mode switch." if en else "De modusschakelaar."))
    rows = [
        ("Esc", "Stop sending at once" if en else "Zenden meteen stoppen"),
        ("Enter", "Contest: send the right message for the moment (Run: CQ, exchange, AGN?, then TU and log)" if en else "Contest: het juiste bericht voor dat moment zenden (Run: CQ, uitwisseling, AGN?, dan TU en loggen)"),
        ("Space", "Contest: next field in the entry line" if en else "Contest: volgend veld in de invoerlijn"),
        ("F1 to F8", "Contest: CQ (S&amp;P: your call), exchange, TU and log, your call, his call, QSO B4, NR?, AGN" if en else "Contest: CQ (S&amp;P: je roepletters), uitwisseling, TU en loggen, je roepletters, zijn roepletters, QSO B4, NR?, AGN"),
        ("Ctrl+R", "Contest: switch between Run and S&amp;P" if en else "Contest: wisselen tussen Run en S&amp;P"),
        ("Ctrl+W", "Contest: clear the entry line" if en else "Contest: de invoerlijn wissen"),
        ("Ctrl+F6, Ctrl+F7", "Contest: decoder Strongest and AFC" if en else "Contest: decoder Strongest en AFC"),
    ]
    h.append(f'<h2 id="keys"><span class="letter">K</span>{toc[1][1]}</h2><table class="keys"><tr><th>{"Key" if en else "Toets"}</th><th>{"What it does" if en else "Wat het doet"}</th></tr>'
             + ''.join(f'<tr><td><kbd>{k}</kbd></td><td>{v}</td></tr>' for k, v in rows) + '</table>')
    h.append(f'<h2 id="faq"><span class="letter">?</span>{toc[2][1]}</h2>' + ''.join(f'<details><summary>{q}</summary><div class="a">{a}</div></details>' for q, a in faq_items(L)))
    h.append(f'<h2 id="contest"><span class="letter">C</span>{toc[3][1]}</h2>' + ('''
<p>Switch contest mode on and choose a contest profile. The entry line takes callsign, report, serial number and the extra field of the profile. The serial number goes up when you log, not when you send. A red DUPE shows the same call on the same band, and logging stays possible.</p>
<p>Test mode (orange) writes to a separate test log that never counts for dupes and never reaches the export.</p>''' if en else '''
<p>Zet de contestmodus aan en kies een contestprofiel. De invoerlijn neemt roepletters, rapport, volgnummer en het extra veld van het profiel. Het volgnummer gaat omhoog als je logt, niet als je het zendt. Een rode DUPE toont dezelfde roepletters op dezelfde band, en loggen blijft mogelijk.</p>
<p>De testmodus (oranje) schrijft naar een apart testlog dat nooit meetelt voor dupes en nooit in de export komt.</p>'''))
    h.append(f'<h2 id="portable"><span class="letter">P</span>{toc[4][1]}</h2>' + ('''
<p>Choose the profile Home or Portable in the status bar. For an activation (WWFF/ONFF, POTA, SOTA) fill in the reference and the grid. Both are written into every QSO, and a check window before the ADIF export catches missing grids or bad references.</p>''' if en else '''
<p>Kies het profiel Home of Portable in de statusbalk. Voor een activatie (WWFF/ONFF, POTA, SOTA) vul je de referentie en het grid in. Beide komen in elk QSO, en een controlevenster voor de ADIF-export vangt ontbrekende grids of foute referenties op.</p>'''))
    h.append(f'<h2 id="practice"><span class="letter">O</span>{toc[5][1]}</h2>' + ('''
<p>PRACTICE decodes a simulated band and plays it on your computer. Nothing is transmitted, so turn the AF knob of the radio down. Choose an exercise, a speed range and a signal strength, and see your score at the end. Turn decoder help off to copy by ear.</p>''' if en else '''
<p>PRACTICE decodeert een gesimuleerde band en speelt die af op je computer. Er wordt niets gezonden, dus draai de AF-knop van de radio laag. Kies een oefening, een snelheidsbereik en een signaalsterkte, en bekijk op het einde je score. Zet de hulp van de decoder uit om op het gehoor te kopiëren.</p>''')
        + fig(img(L, "practice", "The practice panel" if en else "Het oefenpaneel"), "The practice panel." if en else "Het oefenpaneel."))
    h.append('</article></div>')
    return ''.join(h)


def index_body(L):
    en = L == "en"
    t = T[L]
    h = []
    # hero
    h.append(f'<section class="hero"><canvas id="wf" aria-hidden="true"></canvas><div class="wrap"><div>'
             f'<div class="eyebrow"><i></i>{"CW station app" if en else "CW-station-app"}</div><h1>{t["h1"]}</h1><p class="lead">{t["lead"]}</p>'
             f'<div class="cta"><a class="btn primary" data-dl href="{REL}">{t["dl"]} <small data-version></small></a><a class="btn" href="{{INSTALL}}">{t["guide"]}</a></div>'
             f'<div class="fine">{"".join(f"<span>{x}</span>" for x in t["fine"])}</div></div>'
             f'<div class="demo" id="demo" data-script=\'{escape(json.dumps(t["demo"]), quote=True)}\'><div class="bar"><span>●</span><span>{t["demo_bar"]}</span><span class="meter"><i></i></span></div><div class="feed" aria-live="off"></div></div>'
             f'</div></section>')
    # features
    h.append(f'<section class="s" id="features"><div class="wrap"><div class="kicker">{t["feat_k"]}</div><h2>{t["feat_h"]}</h2><p class="sub">{t["feat_s"]}</p><div class="grid">'
             + ''.join(f'<div class="card"><div class="ico">{i}</div><h3>{a}</h3><p>{b}</p></div>' for i, a, b in t["feats"]) + '</div></div></section>')
    # screen
    h.append(f'<section class="s alt"><div class="wrap two"><div><div class="kicker">{t["two_k"]}</div><h2>{t["two_h"]}</h2><p class="sub">{t["two_p"]}</p><ul class="checks">'
             + ''.join(f'<li>{x}</li>' for x in t["two_l"]) + f'</ul></div><div class="shot-frame">{img(L, "main", "The CWLD main screen" if en else "Het hoofdscherm van CWLD")}</div></div></section>')
    # modes
    h.append(f'<section class="s"><div class="wrap"><div class="kicker">{t["modes_k"]}</div><h2>{t["modes_h"]}</h2><p class="sub"></p><div class="modes">'
             + ''.join(f'<div class="mode {c}"><h3>{n}</h3><p>{d}</p></div>' for c, n, d in t["modes"]) + '</div></div></section>')
    # toy
    h.append(f'<section class="s alt"><div class="wrap"><div class="kicker">{t["toy_k"]}</div><h2>{t["toy_h"]}</h2><p class="sub">{t["toy_p"]}</p>'
             f'<div class="toy" id="toy"><input type="text" value="{t["toy_ph"]}" maxlength="24" aria-label="text" spellcheck="false"><div class="morse" aria-live="polite"></div>'
             f'<div class="row"><button class="btn amber" type="button" data-stop="{t["toy_stop"]}">{t["toy_play"]}</button><label>{t["toy_label"]} <input type="range" min="8" max="35" value="18"> <output class="wpm">18</output> WPM</label></div></div></div></section>')
    # download
    h.append(f'<section class="s" id="download"><div class="wrap"><div class="kicker">{t["dl_k"]}</div><h2>{t["dl_h"]}</h2><p class="sub"></p><div class="dl">'
             f'<div class="box glow"><div class="muted">{t["dl_ver"]}</div><div class="ver"><span data-version>...</span></div>'
             f'<p><a class="btn primary" data-dl href="{REL}">{t["dl_btn"]}</a></p>'
             f'<dl class="facts" data-live hidden><dt>{t["dl_file"]}</dt><dd data-file></dd><dt>{t["dl_size"]}</dt><dd data-size></dd><dt>{t["dl_date"]}</dt><dd data-date></dd><dt>{t["dl_sha"]}</dt><dd class="hash" data-hash data-nohash="{t["dl_nohash"]}"></dd></dl>'
             f'<p data-live hidden><a data-notes href="{REL}">{t["dl_notes"]}</a></p>'
             f'<p class="fine" data-fail hidden>{t["dl_fail"]} <a href="{REL}">{t["dl_fail_a"]}</a></p>'
             f'<div class="warn"><b>SmartScreen</b>{t["dl_smart"]}</div></div>'
             f'<div class="box"><h3>{t["req_h"]}</h3><ul class="reqs">' + ''.join(f'<li><b>{a}</b><span>{b}</span></li>' for a, b in t["reqs"]) + '</ul></div></div></div></section>')
    # promo
    h.append(f'<section class="s alt"><div class="wrap"><div class="promo card"><h2>{t["promo_h"]}</h2><p>{t["promo_p"]}</p><p><a class="btn" href="{TRAINER}">{t["promo_a"]} ↗</a> <a class="btn" href="{OWN}">{t["own_a"]} ↗</a></p></div></div></section>')
    return ''.join(h)


def changelog_page(L):
    t = T[L]
    return f'<div class="wrap" style="padding:48px 0 96px"><p class="fine">{t["c_note"]} <a href="{GH}/releases">GitHub Releases</a></p><div class="changelog">{changelog_html(L)}</div></div>'


# ------------------------------------------------------------------ layout
def layout(L, page, title, desc, body, head=None):
    t = T[L]
    other = "nl" if L == "en" else "en"
    depth = 0 if L == "en" else 1
    R = "../" * depth

    def url(lang, pg):
        base = "" if lang == "en" else "nl/"
        return os.path.relpath(base + pg + ".html", start=("" if L == "en" else "nl")) if True else ""

    def link(pg):
        return pg + ".html"

    sw = os.path.relpath(("" if other == "en" else "nl/") + page + ".html", start=("" if L == "en" else "nl")).replace("\\", "/")
    nav = ''.join(f'<a href="{link(p)}"{" aria-current=\"page\"" if p == page else ""}>{NAV[L][p]}</a>' for p in PAGES)
    nav += f'<a class="ext" href="{GH}">GitHub</a><a class="ext" href="{OWN}">ON3VZ</a>'
    langs = (f'<span class="lang"><a href="{sw if L == "nl" else page + ".html"}" hreflang="en"{" aria-current=\"true\"" if L == "en" else ""}>EN</a>'
             f'<a href="{sw if L == "en" else page + ".html"}" hreflang="nl"{" aria-current=\"true\"" if L == "nl" else ""}>NL</a></span>')
    guides = f'<a href="{link("install")}">{NAV[L]["install"]}</a><a href="{link("help")}">{NAV[L]["help"]}</a><a href="{link("changelog")}">{NAV[L]["changelog"]}</a>'
    flinks = ''.join(f'<a href="{u}">{n}</a>' for n, u in t["f_links"])
    more = f'<a href="{OWN}">on3vz.github.io</a><a href="{TRAINER}">{"CWLD morse trainer" if L == "en" else "CWLD-morsetrainer"}</a>'
    ph = (head or "")
    html = f'''<!doctype html>
<html lang="{L}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{escape(title)}</title><meta name="description" content="{escape(desc)}">
<meta name="theme-color" content="#070b10"><link rel="icon" href="{R}favicon.svg" type="image/svg+xml">
<meta property="og:title" content="{escape(title)}"><meta property="og:description" content="{escape(desc)}">
<link rel="stylesheet" href="{R}assets/style.css">{ph}</head><body>
<a class="skip" href="#main">{"Skip to content" if L == "en" else "Naar de inhoud"}</a>
<header class="top"><div class="wrap"><a class="logo" href="{link("index")}">{LOGO}<span>CW<b>LD</b></span></a>
<button class="menu-btn" aria-expanded="false" aria-label="Menu">☰</button><nav class="main">{nav}{langs}</nav></div></header>
<main id="main">{{PAGEHEAD}}{body}</main>
<footer><div class="wrap"><div><div class="logo">{LOGO}<span>CW<b>LD</b></span></div><p>{t["f_about"]}</p></div>
<div><h4>{t["f_proj"]}</h4>{flinks}</div><div><h4>{t["f_guides"]}</h4>{guides}</div><div><h4>{t["f_more"]}</h4>{more}</div>
<div class="made"><span>{t["f_made"]}</span><span class="ontheair">{t["f_air"]}</span></div></div></footer>
<script src="{R}assets/app.js" defer></script></body></html>'''
    pagehead = ""
    if page != "index":
        pagehead = f'<div class="page-head"><div class="wrap"><div class="eyebrow">CWLD</div><h1>{escape(title.split(" | ")[0])}</h1><p>{desc}</p></div></div>'
    html = html.replace("{PAGEHEAD}", pagehead)
    html = html.replace("{R}", R).replace("{INSTALL}", link("install")).replace("{HELP}", link("help"))
    return html


def main():
    if os.path.isdir(OUT):
        shutil.rmtree(OUT)
    os.makedirs(os.path.join(OUT, "nl"))
    shutil.copytree(os.path.join(SRC, "assets"), os.path.join(OUT, "assets"))
    shutil.copytree(os.path.join(SRC, "img"), os.path.join(OUT, "img"))
    open(os.path.join(OUT, "favicon.svg"), "w").write(LOGO.replace("<svg ", '<svg xmlns="http://www.w3.org/2000/svg" ', 1))
    open(os.path.join(OUT, ".nojekyll"), "w").write("")
    for L in ("en", "nl"):
        t = T[L]
        base = os.path.join(OUT, "" if L == "en" else "nl")
        content = {
            "index": ("CWLD | " + t["tag"], t["tag"], index_body(L)),
            "install": (t["i_title"] + " | CWLD", t["i_desc"], install_body(L)),
            "help": (t["h_title"] + " | CWLD", t["h_desc"], help_body(L)),
            "changelog": (t["c_title"] + " | CWLD", t["c_desc"], changelog_page(L)),
        }
        for pg, (title, desc, body) in content.items():
            open(os.path.join(base, pg + ".html"), "w", encoding="utf-8").write(layout(L, pg, title, desc, body))
    # svgs also as files so they can be linked or reused
    os.makedirs(os.path.join(OUT, "img"), exist_ok=True)
    print("built", OUT)


if __name__ == "__main__":
    main()
