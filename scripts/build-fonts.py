"""Builds the fonts bundled in assets/fonts.

Run with: uv run --with fonttools scripts/build-fonts.py

- Roboto Flex: every axis pinned to its default except weight, which
  iced's text stack drives, and a static bold instance beside it, as iced
  picks a face only at a weight it was registered with, and the variable
  font registers as regular alone. Without it, bold text, as in Markdown,
  falls back to a system font (a monospaced one on macOS).
- Material Symbols Rounded: static outlined and filled instances at 24 px
  optical size, subset to the icons listed in src/icon.rs.
"""

import io
import math
import re
import sys
import tempfile
import urllib.request
from pathlib import Path

from fontTools.pens.transformPen import TransformPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.subset import Options, Subsetter
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

FONTS_COMMIT = "23e54b51ddffbc7713c583748e3bd86f62b1fa4a"
ICONS_COMMIT = "bd8cb85bd4bad964fe6918f79665bb40c3a8efef"
ROBOTO_FLEX = (
    f"https://github.com/google/fonts/raw/{FONTS_COMMIT}/ofl/robotoflex/"
    "RobotoFlex%5BGRAD,XOPQ,XTRA,YOPQ,YTAS,YTDE,YTFI,YTLC,YTUC,opsz,slnt,wdth,wght%5D.ttf"
)
ROBOTO_FLEX_LICENSE = f"https://github.com/google/fonts/raw/{FONTS_COMMIT}/ofl/robotoflex/OFL.txt"
SYMBOLS = (
    f"https://github.com/google/material-design-icons/raw/{ICONS_COMMIT}/variablefont/"
    "MaterialSymbolsRounded%5BFILL,GRAD,opsz,wght%5D.ttf"
)
SYMBOLS_LICENSE = f"https://github.com/google/material-design-icons/raw/{ICONS_COMMIT}/LICENSE"

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets/fonts"
ICON_TABLE = ROOT / "src/icon.rs"

# Glyphs the font lacks, drawn by transforming another glyph:
# (made-up name in icon.rs, source codepoint, transform).
DERIVED = {
    "flip rotated a quarter turn": (0xE3E8, "quarter_turn"),
    # Their arrows pointing left, in, and right, out, rather than up and
    # down.
    "input_circle rotated a quarter turn": (0xF71A, "quarter_turn"),
    "output_circle rotated a quarter turn": (0xF70E, "quarter_turn"),
    # A network bridge that is a Wi-Fi access point too: the hub, smaller,
    # under two arcs that rise from its top node.
    "hub under Wi-Fi arcs": (0xE9F4, "wifi_arcs"),
}

# The hub under Wi-Fi arcs, in font units: the hub at this scale, moved
# so its top node centres on the arcs; and the arcs' radii, the width of
# their strokes, and the angles they span either side of straight up.
HUB_SCALE = 0.62
HUB_SHIFT = (182.4, -4.82)
ARC_RADII = (190, 320)
ARC_WIDTH = 58
ARC_SPAN = 45


def fetch(url: str, into: Path) -> Path:
    path = into / url.rsplit("/", 1)[-1].replace("%5B", "[").replace("%5D", "]")
    with urllib.request.urlopen(url) as response:
        path.write_bytes(response.read())
    return path


def icons() -> list[tuple[int, str]]:
    table = ICON_TABLE.read_text()
    entries = re.findall(r"^\s+\w+ = (0x[0-9a-f]+), \"([^\"]+)\";", table, re.MULTILINE)
    if not entries:
        sys.exit(f"no icons found in {ICON_TABLE}")
    return [(int(codepoint, 16), name) for codepoint, name in entries]


def rename(font: TTFont, family: str) -> None:
    postscript = family.replace(" ", "")
    for record in font["name"].names:
        if record.nameID in (1, 16):
            record.string = family
        elif record.nameID == 4:
            record.string = family
        elif record.nameID == 6:
            record.string = postscript


def arc_contour(pen, centre: tuple[float, float], radius: float) -> None:
    """Draws an arc around `centre`, ARC_WIDTH wide with round ends, from
    ARC_SPAN degrees left of straight up to as far right, as one closed
    contour, clockwise, its curves quadratic, none over 22.5 degrees."""
    cx, cy = centre
    half = ARC_WIDTH / 2
    low, high = 90 - ARC_SPAN, 90 + ARC_SPAN

    def point(at, r, degrees):
        return (at[0] + r * math.cos(math.radians(degrees)), at[1] + r * math.sin(math.radians(degrees)))

    def curve(at, r, start, end):
        steps = max(1, math.ceil(abs(end - start) / 22.5))
        step = (end - start) / steps
        for n in range(steps):
            a0 = start + n * step
            middle = a0 + step / 2
            control = point(at, r / math.cos(math.radians(step / 2)), middle)
            pen.qCurveTo(control, point(at, r, a0 + step))

    left = point((cx, cy), radius, high)
    right = point((cx, cy), radius, low)
    pen.moveTo(point((cx, cy), radius + half, high))
    # The outer edge, left to right, then round the right end inward.
    curve((cx, cy), radius + half, high, low)
    curve(right, half, low, low - 180)
    # The inner edge, right to left, then round the left end outward.
    curve((cx, cy), radius - half, low, high)
    curve(left, half, high + 180, high)
    pen.closePath()


def add_derived(font: TTFont, codepoint: int, source: int, transform: str) -> None:
    cmap = font.getBestCmap()
    if codepoint in cmap:
        sys.exit(f"U+{codepoint:X} is already used in the icon font")
    glyph_set = font.getGlyphSet()
    source_name = cmap[source]
    width, _ = font["hmtx"][source_name]
    ascent = font["hhea"].ascent
    descent = font["hhea"].descent
    pen = TTGlyphPen(glyph_set)
    if transform == "quarter_turn":
        # Rotate 90 degrees about the centre of the icon's em box.
        centre_x = width / 2
        centre_y = (ascent + descent) / 2
        matrix = (0, 1, -1, 0, centre_x + centre_y, centre_y - centre_x)
        glyph_set[source_name].draw(TransformPen(pen, matrix))
    elif transform == "wifi_arcs":
        dx, dy = HUB_SHIFT
        glyph_set[source_name].draw(TransformPen(pen, (HUB_SCALE, 0, 0, HUB_SCALE, dx, dy)))
        # The hub's top node, centred at (480, 840) in its own units.
        top = (480 * HUB_SCALE + dx, 840 * HUB_SCALE + dy)
        for radius in ARC_RADII:
            arc_contour(pen, top, radius)
    else:
        sys.exit(f"unknown transform {transform}")
    name = f"derived{codepoint:x}"
    font["glyf"][name] = pen.glyph()
    font["hmtx"][name] = (width, 0)
    for table in font["cmap"].tables:
        if table.isUnicode():
            table.cmap[codepoint] = name


def build_symbols(source: Path, fill: int, family: str, out: Path) -> None:
    font = TTFont(source)
    instantiateVariableFont(font, {"FILL": fill, "GRAD": 0, "opsz": 24, "wght": 400}, inplace=True)
    wanted = []
    for codepoint, name in icons():
        if name in DERIVED:
            base, transform = DERIVED[name]
            add_derived(font, codepoint, base, transform)
        wanted.append(codepoint)
    # Reload so the subsetter sees the added glyphs.
    buffer = io.BytesIO()
    font.save(buffer)
    buffer.seek(0)
    font = TTFont(buffer)
    missing = [f"U+{c:X}" for c in wanted if c not in font.getBestCmap()]
    if missing:
        sys.exit(f"icons missing from the font: {', '.join(missing)}")
    options = Options()
    options.layout_features = []
    options.name_IDs = ["*"]
    options.notdef_outline = True
    subsetter = Subsetter(options)
    subsetter.populate(unicodes=wanted)
    subsetter.subset(font)
    rename(font, family)
    font.save(out)


def build_roboto_flex(source: Path, out: Path) -> None:
    font = TTFont(source)
    pinned = {axis.axisTag: axis.defaultValue for axis in font["fvar"].axes if axis.axisTag != "wght"}
    instantiateVariableFont(font, pinned, inplace=True)
    font.save(out)


def build_roboto_flex_bold(variable: Path, out: Path) -> None:
    font = TTFont(variable)
    instantiateVariableFont(font, {"wght": 700}, inplace=True, updateFontNames=False)
    font["OS/2"].usWeightClass = 700
    # Bold, not regular, in the style bits.
    font["OS/2"].fsSelection = (font["OS/2"].fsSelection & ~0x40) | 0x20
    font["head"].macStyle |= 0x1
    names = {2: "Bold", 4: "Roboto Flex Bold", 6: "RobotoFlex-Bold", 17: "Bold"}
    for record in font["name"].names:
        if record.nameID in names:
            record.string = names[record.nameID]
    font.save(out)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as scratch:
        scratch = Path(scratch)
        build_roboto_flex(fetch(ROBOTO_FLEX, scratch), OUT / "RobotoFlex.ttf")
        build_roboto_flex_bold(OUT / "RobotoFlex.ttf", OUT / "RobotoFlexBold.ttf")
        (OUT / "OFL.txt").write_bytes(fetch(ROBOTO_FLEX_LICENSE, scratch).read_bytes())
        symbols = fetch(SYMBOLS, scratch)
        build_symbols(symbols, 0, "Material Symbols Rounded", OUT / "MaterialSymbolsRounded.ttf")
        build_symbols(symbols, 1, "Material Symbols Rounded Filled", OUT / "MaterialSymbolsRoundedFilled.ttf")
        (OUT / "LICENSE-MaterialSymbols.txt").write_bytes(fetch(SYMBOLS_LICENSE, scratch).read_bytes())
    for path in sorted(OUT.iterdir()):
        print(f"{path.stat().st_size:>9}  {path.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
