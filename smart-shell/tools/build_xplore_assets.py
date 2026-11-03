#!/usr/bin/env python3
"""Build Xplore's static assets from the prototype's sources.

Four jobs, all deterministic and offline:

1. Convert `assets/fonts/InterVariable.woff2` to a TTF that GPUI's font stack
   can load, then cut static instances at the contract's weights. GPUI resolves
   fonts by face weight rather than by variable axis, so a single variable file
   would render every weight at 400.
2. Extract the Solar icons the prototype uses from `assets/icons.js` into
   standalone SVG files, one per (name, variant).
3. Pre-blur and pre-saturate `assets/wallpaper.jpg` into the glass backdrop
   the native shell paints, replacing the browser's `backdrop-filter`.
4. Copy the prototype's thumbnail images the file cards show.
5. Copy the canonical design contract into the app, so the native shell parses
   the same normative source the prototype's CSS is generated from.

Run from the prototype root:
    python3 tools/build_xplore_assets.py
"""

from __future__ import annotations

import io
import json
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROTO = ROOT
OUT = ROOT / "apps" / "Xplore" / "assets"

ICONS_JS = PROTO / "assets" / "icons.js"
WALLPAPER = PROTO / "assets" / "wallpaper.jpg"
THUMBS = PROTO / "assets" / "thumbs"
FONT_WOFF2 = PROTO / "assets" / "fonts" / "InterVariable.woff2"

# The canonical contract. The prototype renders with `assets/tokens.css`,
# which is generated from this file, so the native shell parses the same source
# rather than the older draft copy kept beside the prototype.
CONTRACT_SRC = ROOT.parents[1] / "smart-design" / "smart.toml"
CONTRACT_OUT = ROOT / "apps" / "Xplore" / "contract" / "smart.toml"

FONT_DIR = OUT / "fonts"
FONT_TTF = FONT_DIR / "InterVariable.ttf"
ICON_DIR = OUT / "icons" / "solar"
THUMB_DIR = OUT / "thumbs"
WALLPAPER_GLASS = OUT / "wallpaper-glass.png"
WALLPAPER_MANIFEST = OUT / "wallpaper.json"

# The weights `tokens.toml` asks for, plus the regular weight the shell falls
# back to. GPUI picks a face by its declared weight, so each one becomes its
# own static instance named after the contract's family.
FONT_WEIGHTS = {
    "Regular": 400,
    "Medium": 565,
    "SemiBold": 610,
    "Bold": 645,
    "Body": 510,
}
FONT_FAMILY = "InterVariable"


def _set_name(font, name_id: int, value: str) -> None:
    for record in font["name"].names:
        if record.nameID == name_id:
            record.string = value.encode(record.getEncoding())


def convert_font() -> None:
    from fontTools.ttLib import TTFont
    from fontTools.varLib import instancer

    base = TTFont(io.BytesIO(FONT_WOFF2.read_bytes()))
    base.flavor = None
    FONT_DIR.mkdir(parents=True, exist_ok=True)

    # The full variable font, kept for reference and for any weight the
    # contract adds later.
    base.save(FONT_TTF)

    for style, weight in FONT_WEIGHTS.items():
        font = TTFont(io.BytesIO(FONT_WOFF2.read_bytes()))
        font.flavor = None
        instance = instancer.instantiateVariableFont(
            font, {"wght": weight, "opsz": 14}, inplace=True, updateFontNames=False
        )

        # GPUI reads the weight from the face's OS/2 table and the family from
        # the name table, so both have to say what the instance actually is.
        instance["OS/2"].usWeightClass = weight
        _set_name(instance, 1, FONT_FAMILY)
        _set_name(instance, 2, style)
        _set_name(instance, 4, f"{FONT_FAMILY} {style}")
        _set_name(instance, 6, f"{FONT_FAMILY}-{style}")
        _set_name(instance, 16, FONT_FAMILY)
        _set_name(instance, 17, style)

        path = FONT_DIR / f"{FONT_FAMILY}-{style}.ttf"
        instance.save(path)

    print(
        f"font: {FONT_TTF.name} + {len(FONT_WEIGHTS)} static instances "
        f"({', '.join(str(w) for w in FONT_WEIGHTS.values())})"
    )


SYMBOL_RE = re.compile(r'<symbol id="i-([a-z0-9-]+)" viewBox="([^"]+)">(.*?)</symbol>', re.S)


def extract_icons() -> list[str]:
    source = ICONS_JS.read_text()
    # icons.js assigns a JS string literal; decode the escapes to get raw SVG.
    start = source.index('"') + 1
    end = source.rindex('"')
    svg = source[start:end].replace('\\"', '"').replace("\\\\", "\\")

    symbols = {name: (viewbox, body) for name, viewbox, body in SYMBOL_RE.findall(svg)}
    ICON_DIR.mkdir(parents=True, exist_ok=True)

    written = []
    for name, (viewbox, body) in sorted(symbols.items()):
        path = ICON_DIR / f"{name}.svg"
        # Solar symbols carry their own paint attributes (`fill="currentColor"`).
        # GPUI renders SVG as an alpha mask and tints it, so the wrapper must not
        # add stroke or fill attributes of its own.
        document = (
            f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{viewbox}">'
            f"{body}</svg>"
        )
        path.write_text(document)
        written.append(name)

    print(f"icons: {len(written)} extracted to {ICON_DIR}")
    return written


def build_wallpaper() -> None:
    from PIL import Image, ImageEnhance, ImageFilter

    source = Image.open(WALLPAPER).convert("RGB")

    # The prototype's window uses `backdrop-filter: blur(64px) saturate(158%)`
    # and its sidebar `blur(34px) saturate(146%)`. Both blurs act on the same
    # static wallpaper, so the native shell paints one pre-blurred image.
    glass = source.filter(ImageFilter.GaussianBlur(radius=34))
    glass = ImageEnhance.Color(glass).enhance(1.46)

    OUT.mkdir(parents=True, exist_ok=True)
    glass.save(WALLPAPER_GLASS, format="PNG", optimize=True)

    manifest = {
        "source": str(WALLPAPER.relative_to(ROOT)),
        "glass": str(WALLPAPER_GLASS.relative_to(ROOT)),
        "width": source.width,
        "height": source.height,
        "blur_px": 34,
        "saturation": 1.46,
    }
    WALLPAPER_MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n")
    print(
        f"wallpaper: {WALLPAPER_GLASS} "
        f"({source.width}x{source.height}, blur 34px, saturate 146%)"
    )


def copy_thumbs() -> None:
    if not THUMBS.exists():
        print(f"thumbs: no source directory at {THUMBS}, skipping")
        return

    THUMB_DIR.mkdir(parents=True, exist_ok=True)
    for stale in THUMB_DIR.glob("*"):
        stale.unlink()

    copied = 0
    for path in sorted(THUMBS.iterdir()):
        if path.suffix.lower() not in {".jpg", ".jpeg", ".png", ".webp"}:
            continue
        shutil.copy2(path, THUMB_DIR / path.name)
        copied += 1

    print(f"thumbs: {copied} copied to {THUMB_DIR}")


def copy_contract() -> None:
    if not CONTRACT_SRC.exists():
        print(f"contract: no canonical source at {CONTRACT_SRC}, skipping")
        return

    CONTRACT_OUT.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(CONTRACT_SRC, CONTRACT_OUT)
    print(f"contract: {CONTRACT_OUT} (from {CONTRACT_SRC})")


def main() -> int:
    if not ICONS_JS.exists():
        print(f"missing prototype assets: {PROTO}", file=sys.stderr)
        return 1
    convert_font()
    extract_icons()
    build_wallpaper()
    copy_thumbs()
    copy_contract()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
