#!/usr/bin/env python3
"""Generate assets/daw-icons.js — Solar icon symbols the shell sprite lacks.

Fetches real Solar paths from the Iconify API and appends them to window.__SOLAR__
using the same naming contract as assets/icons.js:

    i-<name>-outline   → solar <name>-outline
    i-<name>-duotone   → solar <name>-line-duotone
    i-<name>-filled    → solar <name>-bold     (Solar's "bold" is the filled weight)

Run:  python3 tools/build_daw_icons.py           (online)
"""
import json, re, urllib.request, pathlib, sys

API = "https://api.iconify.design/solar.json?icons="
OUT = pathlib.Path(__file__).resolve().parent.parent / "assets" / "daw-icons.js"

# Base names every DAW surface needs, beyond the 44 the shell sprite already ships.
ICONS = """
play pause stop record play-circle pause-circle stop-circle record-audio-circle
rewind-back rewind-forward skip-previous skip-next repeat repeat-one shuffle
volume volume-loud volume-cross muted soundwave soundwave-square
music-note-2 music-note-3 music-note-4 vinyl vinyl-record album podcast
microphone microphone-3 slider-vertical slider-horizontal tuning-2 settings
magnet magic-wand magic-wand-3 scissors eraser pen pen-2 reorder grip-vertical
bolt filter graph scale ruler palette target bookmark star lock unlock
cursor cursor-square minus-circle wind hamburger-menu upload-track
danger danger-triangle waterdrop diskette library stream full-screen
quit-full-screen power check-read clipboard folder-open folder-with-files
quarter-circle structures check-square clipboard-check clipboard-list
drop bolt-circle videocamera-record""".split()

VARIANTS = {          # our suffix  → solar suffix
    "-outline": "-outline",
    "-duotone": "-line-duotone",
    "-filled": "-bold",
}


def fetch(names):
    req = urllib.request.Request(API + ",".join(names),
                                 headers={"User-Agent": "Mozilla/5.0"})
    return json.loads(urllib.request.urlopen(req, timeout=90).read().decode())


def main():
    wanted = []
    for base in ICONS:
        for suffix in VARIANTS.values():
            wanted.append(base + suffix)
    # de-dup, keep order
    seen, uniq = set(), []
    for n in wanted:
        if n not in seen:
            seen.add(n)
            uniq.append(n)

    data, missing = {}, []
    for i in range(0, len(uniq), 120):
        chunk = uniq[i:i + 120]
        res = fetch(chunk)
        data.update(res.get("icons", {}))
        missing += res.get("not_found", [])
    if missing:
        print("not available in solar:", ", ".join(sorted(set(missing))))

    symbols, count = [], 0
    for base in ICONS:
        for ours, theirs in VARIANTS.items():
            body = data.get(base + theirs)
            if not body:
                continue
            inner = body.get("body", "")
            # Solar bodies may carry per-icon width/height; the sprite is always 24.
            symbols.append(
                '<symbol id="i-%s%s" viewBox="0 0 24 24">%s</symbol>' % (base, ours, inner))
            count += 1

    js = ("// GENERATED FILE — do not edit. Source: Solar (Iconify), CC BY 4.0 by 480 Design.\n"
          "// Extend the shared shell sprite with the transport, mixer, and edit symbols the\n"
          "// DAW needs. Naming contract matches assets/icons.js (outline / duotone / filled).\n"
          "window.__SOLAR__ = window.__SOLAR__.replace('</defs>',\n"
          "  `%s</defs>`);\n") % "".join(symbols)

    OUT.write_text(js)
    print("wrote %s — %d symbols for %d icons" % (OUT, count, len(ICONS)))


if __name__ == "__main__":
    sys.exit(main())
