"""Schematic illustrations for the install guide. Plain SVG, generated per language.
They are drawings, not screenshots: the real app screens come from site/shots.mjs."""
from html import escape

BG = "#0f1822"
LINE = "#2c4560"
TXT = "#e8eff6"
MUT = "#8ea3b7"
AMB = "#ffb020"
SCO = "#3dffa8"
CYA = "#4cc9ff"


def _t(L, en, nl):
    return en if L == "en" else nl


def badge(x, y, n):
    return (f'<g><circle cx="{x}" cy="{y}" r="17" fill="{AMB}"/><circle cx="{x}" cy="{y}" r="22" fill="none" stroke="{AMB}" stroke-opacity=".35" stroke-width="4"/>'
            f'<text x="{x}" y="{y + 6}" text-anchor="middle" font-family="ui-monospace,Consolas,monospace" font-weight="800" font-size="18" fill="#1a1200">{n}</text></g>')


def text(x, y, s, size=16, fill=TXT, weight=400, anchor="start", mono=False):
    fam = "ui-monospace,Consolas,monospace" if mono else "system-ui,Segoe UI,Roboto,sans-serif"
    return f'<text x="{x}" y="{y}" font-family="{fam}" font-size="{size}" font-weight="{weight}" fill="{fill}" text-anchor="{anchor}">{escape(s)}</text>'


def wrap(w, h, body, label):
    return (f'<svg viewBox="0 0 {w} {h}" role="img" aria-label="{escape(label)}" xmlns="http://www.w3.org/2000/svg">'
            f'<rect width="{w}" height="{h}" fill="{BG}"/>{body}</svg>')


def cable(L):
    b = []
    # PC
    b.append(f'<rect x="60" y="60" width="230" height="150" rx="12" fill="#0a121a" stroke="{LINE}" stroke-width="3"/>')
    b.append(f'<rect x="76" y="76" width="198" height="118" rx="6" fill="#060a0f"/>')
    for i, w in enumerate([150, 110, 170, 90]):
        b.append(f'<rect x="90" y="{92 + i * 20}" width="{w}" height="10" rx="5" fill="{["#12384f", "#1c2c3a", "#12384f", "#1c2c3a"][i]}"/>')
    b.append(f'<path d="M30 226 H320 L304 212 H46 Z" fill="#16232f" stroke="{LINE}" stroke-width="3"/>')
    b.append(text(175, 262, _t(L, "Your Windows PC", "Je Windows-pc"), 18, TXT, 600, "middle"))
    # radio
    b.append(f'<rect x="640" y="70" width="320" height="200" rx="18" fill="#101c28" stroke="{LINE}" stroke-width="3"/>')
    b.append(f'<rect x="662" y="92" width="196" height="116" rx="8" fill="#04100c" stroke="#1f5f45" stroke-width="2"/>')
    b.append(f'<polyline points="670,150 700,148 716,120 724,150 760,146 780,100 790,148 830,150 850,146" fill="none" stroke="{SCO}" stroke-width="2"/>')
    for r in range(4):
        b.append(f'<rect x="670" y="{162 + r * 10}" width="180" height="6" rx="3" fill="#0e3a28" opacity="{0.9 - r * 0.18}"/>')
    b.append(f'<circle cx="905" cy="130" r="30" fill="#0a121a" stroke="{LINE}" stroke-width="3"/><circle cx="905" cy="130" r="4" fill="{MUT}"/>')
    b.append(f'<circle cx="905" cy="222" r="20" fill="#0a121a" stroke="{LINE}" stroke-width="3"/>')
    for i in range(5):
        b.append(f'<rect x="{666 + i * 36}" y="226" width="26" height="14" rx="4" fill="#1c2c3a"/>')
    b.append(text(800, 300, "IC-7300 MK II", 20, TXT, 700, "middle", True))
    # port + cable
    b.append(f'<rect x="628" y="168" width="22" height="30" rx="6" fill="#0a121a" stroke="{AMB}" stroke-width="3"/>')
    b.append(f'<path d="M290 150 C 420 150, 500 190, 628 183" fill="none" stroke="{AMB}" stroke-width="7" stroke-linecap="round"/>')
    b.append(f'<rect x="282" y="140" width="22" height="20" rx="5" fill="{AMB}"/>')
    b.append(text(455, 128, _t(L, "USB-C cable", "USB-C-kabel"), 18, AMB, 700, "middle"))
    b.append(text(455, 232, _t(L, "Use a cable that carries data,", "Gebruik een kabel met data,"), 15, MUT, 400, "middle"))
    b.append(text(455, 252, _t(L, "not one that only charges.", "niet een die enkel laadt."), 15, MUT, 400, "middle"))
    b.append(text(620, 150, _t(L, "USB port (Type-C)", "USB-poort (Type-C)"), 15, TXT, 600, "end"))
    b.append(badge(310, 118, 1))
    b.append(badge(628, 118, 2))
    b.append(text(500, 330, _t(L, "One cable carries control (CI-V) and sound (USB audio).", "Eén kabel voor bediening (CI-V) en geluid (USB-audio)."), 16, MUT, 400, "middle"))
    return wrap(1000, 350, "".join(b), _t(L, "A USB-C cable between the PC and the radio", "Een USB-C-kabel tussen de pc en de radio"))


def _card(x, y, w, h, lines, hi=None, title=None):
    out = [f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="10" fill="#04100c" stroke="#1f5f45" stroke-width="2"/>']
    ty = y + 34
    if title:
        out.append(text(x + w / 2, ty, title, 20, SCO, 800, "middle", True))
        ty += 30
    for i, ln in enumerate(lines):
        yy = ty + i * 30
        if hi is not None and i == hi:
            out.append(f'<rect x="{x + 8}" y="{yy - 21}" width="{w - 16}" height="28" rx="6" fill="{AMB}"/>')
            out.append(text(x + 18, yy, ln, 16, "#1a1200", 800, "start", True))
        else:
            out.append(text(x + 18, yy, ln, 16, "#7fd9b0", 400, "start", True))
    return "".join(out)


def radio_menu(L):
    b = []
    cw, gap, y = 170, 28, 20
    xs = [14 + i * (cw + gap) for i in range(5)]
    cards = [
        (["", _t(L, "press", "druk op")], 1, "MENU"),
        (["...", "SET", "..."], 1, None),
        (["...", "Connectors", "..."], 1, None),
        (["...", "CI-V", "..."], 1, None),
        (["CI-V Baud Rate", "CI-V Address", "CI-V Transceive", "CI-V USB Echo Back"], 2, None),
    ]
    for i, (lines, hi, title) in enumerate(cards):
        h = 190
        if i == 0:
            b.append(_card(xs[i], y, cw, h, [], None, "MENU"))
            b.append(f'<rect x="{xs[i] + 36}" y="{y + 86}" width="98" height="46" rx="10" fill="{AMB}"/>')
            b.append(text(xs[i] + 85, y + 117, "MENU", 20, "#1a1200", 800, "middle", True))
            b.append(text(xs[i] + 85, y + 164, _t(L, "press the button", "druk op de knop"), 14, MUT, 400, "middle"))
        elif i == 4:
            b.append(_card(xs[i], y, cw, h, ["Baud Rate", "Address  B6", "Transceive", "USB Echo Back"], 2))
            b.append(f'<rect x="{xs[i] + cw - 54}" y="{y + 107}" width="40" height="22" rx="6" fill="#04100c"/>')
            b.append(text(xs[i] + cw - 34, y + 123, "ON", 15, SCO, 800, "middle", True))
        else:
            b.append(_card(xs[i], y, cw, h, lines, hi))
        b.append(badge(xs[i] + 22, y - 2 + 4, i + 1) if False else "")
        if i < 4:
            ax = xs[i] + cw + 4
            b.append(f'<path d="M{ax} {y + 95} h{gap - 10} m-8 -7 l8 7 l-8 7" fill="none" stroke="{AMB}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>')
    labels = ["MENU", "SET", "Connectors", "CI-V", "CI-V Transceive = ON"]
    for i, lab in enumerate(labels):
        b.append(badge(xs[i] + cw / 2 - (0 if i < 4 else 0), y + 230, i + 1))
        b.append(text(xs[i] + cw / 2, y + 276, lab, 15 if i == 4 else 17, TXT, 700, "middle", True))
    return wrap(1000, 330, "".join(b), _t(L, "Radio menu path: MENU, SET, Connectors, CI-V, CI-V Transceive ON", "Menupad op de radio: MENU, SET, Connectors, CI-V, CI-V Transceive ON"))


def device_manager(L):
    b = []
    b.append(f'<rect x="40" y="20" width="820" height="270" rx="12" fill="#0a121a" stroke="{LINE}" stroke-width="2"/>')
    b.append(f'<rect x="40" y="20" width="820" height="40" rx="12" fill="#16232f"/><rect x="40" y="44" width="820" height="16" fill="#16232f"/>')
    b.append(text(60, 47, _t(L, "Device Manager", "Apparaatbeheer"), 17, TXT, 600))
    rows = [
        (0, "▸", _t(L, "Audio inputs and outputs", "Audio-ingangen en -uitgangen"), False),
        (0, "▾", _t(L, "Ports (COM & LPT)", "Poorten (COM en LPT)"), False),
    ]
    y = 96
    for ind, arrow, label, hi in rows:
        b.append(text(64 + ind * 24, y, arrow, 16, MUT))
        b.append(text(88 + ind * 24, y, label, 17, TXT, 600))
        y += 38
    b.append(f'<rect x="96" y="{y - 24}" width="720" height="34" rx="8" fill="{AMB}" fill-opacity=".18" stroke="{AMB}" stroke-width="2"/>')
    b.append(text(112, y, "IC-7300MK2 Serial Port A (CI-V) (COM5)", 17, TXT, 700, "start", True))
    b.append(badge(836, y - 7, 1))
    y += 40
    b.append(text(112, y, "IC-7300MK2 Serial Port B (COM6)", 17, TXT, 400, "start", True))
    b.append(badge(836, y - 7, 2))
    b.append(text(64, 262, _t(L, "1 = try this one first.  2 = the other one, if 1 does not answer.  Your COM numbers will differ.", "1 = probeer eerst deze.  2 = de andere, als 1 niet antwoordt.  Jouw COM-nummers zullen anders zijn."), 14, MUT))
    return wrap(900, 310, "".join(b), _t(L, "Device Manager with two ports for the radio", "Apparaatbeheer met twee poorten voor de radio"))


def smartscreen(L):
    b = []
    for k, x0 in enumerate([30, 470]):
        b.append(f'<rect x="{x0}" y="14" width="400" height="320" rx="6" fill="#16407a"/>')
        if k == 0:
            b.append(text(x0 + 24, 70, _t(L, "Windows protected your PC", "Windows heeft uw pc beschermd"), 22, "#ffffff", 700))
            for i, ln in enumerate(_t(L, ["Microsoft Defender SmartScreen prevented an", "unrecognized app from starting. Running this", "app might put your PC at risk."], ["Microsoft Defender SmartScreen heeft voorkomen", "dat een niet-herkende app is gestart. Het", "uitvoeren van deze app kan uw pc in gevaar brengen."])):
                b.append(text(x0 + 24, 110 + i * 24, ln, 15, "#e5eefb"))
            b.append(text(x0 + 24, 218, _t(L, "More info", "Meer info"), 16, "#ffffff", 700))
            b.append(f'<rect x="{x0 + 18}" y="196" width="120" height="32" rx="6" fill="none" stroke="{AMB}" stroke-width="3"/>')
            b.append(badge(x0 + 160, 212, 1))
            b.append(f'<rect x="{x0 + 250}" y="284" width="126" height="34" rx="3" fill="#3f6cb0"/>')
            b.append(text(x0 + 313, 307, _t(L, "Don't run", "Niet uitvoeren"), 15, "#ffffff", 600, "middle"))
        else:
            b.append(text(x0 + 24, 70, _t(L, "Windows protected your PC", "Windows heeft uw pc beschermd"), 22, "#ffffff", 700))
            b.append(text(x0 + 24, 120, _t(L, "App:  CWLD_x.y.z_x64-setup.exe", "App:  CWLD_x.y.z_x64-setup.exe"), 15, "#e5eefb", 400, "start", True))
            b.append(text(x0 + 24, 148, _t(L, "Publisher:  Unknown publisher", "Uitgever:  Onbekende uitgever"), 15, "#e5eefb"))
            b.append(f'<rect x="{x0 + 24}" y="270" width="150" height="38" rx="3" fill="#ffffff"/>')
            b.append(text(x0 + 99, 295, _t(L, "Run anyway", "Toch uitvoeren"), 16, "#16407a", 700, "middle"))
            b.append(f'<rect x="{x0 + 18}" y="264" width="162" height="50" rx="6" fill="none" stroke="{AMB}" stroke-width="3"/>')
            b.append(badge(x0 + 206, 289, 2))
            b.append(f'<rect x="{x0 + 250}" y="270" width="126" height="38" rx="3" fill="#3f6cb0"/>')
            b.append(text(x0 + 313, 295, _t(L, "Don't run", "Niet uitvoeren"), 15, "#ffffff", 600, "middle"))
    b.append(text(450, 360, _t(L, "Schematic of the Windows message, the wording on your PC may differ a little.", "Schema van de Windows-melding, de tekst op je pc kan een beetje verschillen."), 13, MUT, 400, "middle"))
    return wrap(900, 376, "".join(b), _t(L, "The SmartScreen message: click More info, then Run anyway", "De SmartScreen-melding: klik op Meer info en dan Toch uitvoeren"))


def mic(L):
    b = []
    b.append(f'<rect x="30" y="20" width="640" height="240" rx="14" fill="#101c28" stroke="{LINE}" stroke-width="2"/>')
    b.append(text(56, 62, _t(L, "Privacy: Microphone", "Privacy: Microfoon"), 22, TXT, 700))
    rows = _t(L, ["Microphone access", "Let apps access your microphone", "Let desktop apps access your microphone"], ["Toegang tot de microfoon", "Apps toegang geven tot uw microfoon", "Desktop-apps toegang geven tot uw microfoon"])
    for i, r in enumerate(rows):
        y = 112 + i * 52
        b.append(text(56, y + 6, r, 17, TXT))
        b.append(f'<rect x="570" y="{y - 14}" width="62" height="30" rx="15" fill="{SCO}"/><circle cx="617" cy="{y + 1}" r="11" fill="#04180f"/>')
        b.append(text(556, y + 6, _t(L, "On", "Aan"), 15, SCO, 700, "end"))
    b.append(badge(650, 112, 1) if False else "")
    return wrap(700, 280, "".join(b), _t(L, "Windows microphone privacy switches turned on", "Windows-privacyschakelaars voor de microfoon aan"))


def qmx_menu(L):
    b = []
    b.append(_card(40, 20, 330, 190, [], None, "QMX"))
    b.append(text(60, 100, "Audio", 18, "#7fd9b0", 700, "start", True))
    b.append(f'<rect x="52" y="116" width="306" height="30" rx="6" fill="{AMB}"/>')
    b.append(text(64, 137, "Sidetone volume   99", 16, "#1a1200", 800, "start", True))
    b.append(f'<rect x="52" y="152" width="306" height="30" rx="6" fill="{AMB}"/>')
    b.append(text(64, 173, "Sidetone abs/rel  Relative", 16, "#1a1200", 800, "start", True))
    b.append(badge(392, 131, 1))
    b.append(badge(392, 167, 2))
    b.append(text(430, 138, _t(L, "high, so CWLD can hear your sidetone", "hoog, zodat CWLD je zijtoon hoort"), 16, TXT))
    b.append(text(430, 174, _t(L, "level follows the volume", "niveau volgt het volume"), 16, TXT))
    return wrap(800, 230, "".join(b), _t(L, "QMX menu: Audio, Sidetone volume high and Sidetone abs/rel Relative", "QMX-menu: Audio, Sidetone volume hoog en Sidetone abs/rel Relative"))


def flow(L):
    b = []
    b.append(f'<rect x="20" y="70" width="210" height="190" rx="16" fill="#101c28" stroke="{LINE}" stroke-width="3"/>')
    b.append(f'<rect x="40" y="92" width="170" height="86" rx="8" fill="#04100c" stroke="#1f5f45" stroke-width="2"/>')
    b.append(f'<polyline points="48,150 70,148 82,124 90,150 120,146 136,108 146,148 180,150 202,146" fill="none" stroke="{SCO}" stroke-width="2"/>')
    b.append(text(125, 214, _t(L, "Your radio", "Je radio"), 18, TXT, 700, "middle"))
    b.append(text(125, 238, "IC-7300 MK II / QMX", 14, MUT, 400, "middle", True))
    b.append(f'<path d="M232 130 H388" stroke="{AMB}" stroke-width="5"/><path d="M232 130 l12 -9 m-12 9 l12 9" stroke="{AMB}" stroke-width="5" fill="none"/><path d="M388 130 l-12 -9 m12 9 l-12 9" stroke="{AMB}" stroke-width="5" fill="none"/>')
    b.append(text(310, 112, "USB-C", 16, AMB, 700, "middle", True))
    b.append(text(310, 158, _t(L, "control + sound", "bediening + geluid"), 14, MUT, 400, "middle"))
    b.append(f'<rect x="390" y="40" width="440" height="250" rx="16" fill="#0d2019" stroke="#1f5f45" stroke-width="3"/>')
    b.append(text(610, 76, "CWLD", 22, SCO, 800, "middle", True))
    items = [
        (_t(L, "Radio control", "Radio-bediening"), "CI-V / CAT"),
        (_t(L, "Listens to the audio", "Luistert naar het geluid"), _t(L, "USB audio in", "USB-audio in")),
        (_t(L, "Decoder", "Decoder"), _t(L, "shows how sure it is", "toont hoe zeker hij is")),
        (_t(L, "Logbook", "Logboek"), _t(L, "ADIF export", "ADIF-export")),
    ]
    for i, (a, c) in enumerate(items):
        x = 410 + (i % 2) * 210
        y = 96 + (i // 2) * 92
        b.append(f'<rect x="{x}" y="{y}" width="200" height="78" rx="10" fill="#101c28" stroke="{LINE}" stroke-width="2"/>')
        b.append(text(x + 14, y + 32, a, 16, TXT, 700))
        b.append(text(x + 14, y + 56, c, 14, MUT))
    return wrap(850, 310, "".join(b), _t(L, "How CWLD connects to the radio", "Hoe CWLD met de radio verbindt"))
