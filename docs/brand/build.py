"""Build Ember's logo files: the Spark mark, the app icon, the lockups and the
README banner, as plain SVG paths (the wordmark outlined, so no font is needed
to display them).

Not part of the build. To regenerate:

    pip install fonttools brotli uharfbuzz
    python docs/brand/build.py SpaceGrotesk[wght].ttf docs/brand

The font is Space Grotesk (SIL Open Font License 1.1), from Google Fonts; the
wordmark is its Bold (wght 700), tracked -0.045 em.
"""
import io
import math
import os
import sys

import uharfbuzz as hb
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

FONT, OUT = sys.argv[1], sys.argv[2]
os.makedirs(OUT, exist_ok=True)

RED, BLACK, WHITE, LINE = "#E0241B", "#0F0F10", "#FAFAF7", "#2A2A2E"
R2 = math.sqrt(2)


def f(x):
    s = f"{x:.3f}".rstrip("0").rstrip(".")
    return "0" if s == "-0" else s


# ---- the mark ---------------------------------------------------------------
# A diamond of half-diagonal D centred on (50, 50) in a 100-unit square, its
# corners rounded by RX; split across its horizontal diagonal by a SEAM, red
# (the ember) above and coal below; and a spark of half-diagonal Ds on the
# diamond's up-right diagonal, SGAP (twice the seam) clear of its edge, with
# corners rounded in the same proportion.
D, RX, SEAM, Ds, SGAP = 42.0, 5.0, 4.5, 7.5, 9.0


def rounded_diamond(cx, cy, h, r):
    """A closed path for a square rotated 45 degrees, corners rounded by r."""
    k = r / R2          # tangent point offset from the arc centre
    c = r * R2          # arc centre offset from the vertex
    top, right, bottom, left = (cx, cy - h), (cx + h, cy), (cx, cy + h), (cx - h, cy)
    p = []
    # start just after the left corner, going clockwise
    p.append(f"M{f(left[0] + c - k)} {f(left[1] - k)}")
    p.append(f"L{f(top[0] - k)} {f(top[1] + c - k)}")
    p.append(f"A{f(r)} {f(r)} 0 0 1 {f(top[0] + k)} {f(top[1] + c - k)}")
    p.append(f"L{f(right[0] - c + k)} {f(right[1] - k)}")
    p.append(f"A{f(r)} {f(r)} 0 0 1 {f(right[0] - c + k)} {f(right[1] + k)}")
    p.append(f"L{f(bottom[0] + k)} {f(bottom[1] - c + k)}")
    p.append(f"A{f(r)} {f(r)} 0 0 1 {f(bottom[0] - k)} {f(bottom[1] - c + k)}")
    p.append(f"L{f(left[0] + c - k)} {f(left[1] + k)}")
    p.append(f"A{f(r)} {f(r)} 0 0 1 {f(left[0] + c - k)} {f(left[1] - k)}Z")
    return "".join(p)


def half(upper):
    """The ember (upper) or coal (lower) half of the diamond, as one path."""
    cx = cy = 50.0
    k, c = RX / R2, RX * R2
    s = -1 if upper else 1                      # direction away from the seam
    y0 = cy + s * SEAM / 2                      # the cut
    # where the cut meets the left and right corner arcs
    lx, rx_ = cx - D + c, cx + D - c           # arc centres
    dx = math.sqrt(RX * RX - (SEAM / 2) ** 2)
    sweep = 1 if upper else 0
    tip_y = cy + s * D
    return "".join([
        f"M{f(lx - dx)} {f(y0)}",
        f"A{f(RX)} {f(RX)} 0 0 {sweep} {f(lx - k)} {f(cy + s * k)}",
        f"L{f(cx - k)} {f(tip_y - s * (c - k))}",
        f"A{f(RX)} {f(RX)} 0 0 {sweep} {f(cx + k)} {f(tip_y - s * (c - k))}",
        f"L{f(rx_ + k)} {f(cy + s * k)}",
        f"A{f(RX)} {f(RX)} 0 0 {sweep} {f(rx_ + dx)} {f(y0)}Z",
    ])


def spark_path():
    dist = D / R2 + SGAP + Ds / R2
    sx, sy = 50 + dist / R2, 50 - dist / R2
    return rounded_diamond(sx, sy, Ds, RX * Ds / D)


EMBER, COAL, SPARK = half(True), half(False), spark_path()


def mark(coal, tx=0.0, ty=0.0, scale=1.0):
    t = f' transform="translate({f(tx)} {f(ty)}) scale({f(scale)})"' if (tx or ty or scale != 1) else ""
    return (f'<g{t}><path fill="{RED}" d="{EMBER}"/><path fill="{coal}" d="{COAL}"/>'
            f'<path fill="{RED}" d="{SPARK}"/></g>')


# The mark's ink: the rounded tips pull in by RX*(sqrt2 - 1).
INSET = 50 - D + RX * (R2 - 1)


# ---- the wordmark -----------------------------------------------------------
def outline(font_path, text, wght, tracking):
    tt = TTFont(font_path)
    tt = instancer.instantiateVariableFont(tt, {"wght": wght})
    buf = io.BytesIO()
    tt.flavor = None
    tt.save(buf)
    data = buf.getvalue()
    face = hb.Face(data)
    font = hb.Font(face)
    b = hb.Buffer()
    b.add_str(text)
    b.guess_segment_properties()
    hb.shape(font, b, {"kern": True, "liga": True})
    gs = tt.getGlyphSet()
    order = tt.getGlyphOrder()
    pen = SVGPathPen(gs)
    x = 0
    n = len(b.glyph_infos)
    for i, (info, pos) in enumerate(zip(b.glyph_infos, b.glyph_positions)):
        name = order[info.codepoint]
        # flip y: font units are y-up; SVG is y-down, baseline at 0
        gs[name].draw(TransformPen(pen, (1, 0, 0, -1, x + pos.x_offset, -pos.y_offset)))
        x += pos.x_advance + (tracking if i < n - 1 else 0)
    upem = tt["head"].unitsPerEm
    return pen.getCommands(), x, upem


WORD, WORD_W, UPEM = outline(FONT, "ember", 700, -45)

# Lockup, in units where the font size is 100 (1 unit = 10 font units).
F = 100.0
H = 1.16 * F                  # the mark's box
GAP = 0.268 * F               # box edge to the word's origin
MID = 0.31 * F                # the mark's centre above the baseline
SC = F / UPEM


def lockup(ink, coal, ox, oy):
    """The lockup with the baseline at oy and the mark's box starting at ox."""
    return (mark(coal, ox, oy - MID - H / 2, H / 100)
            + f'<path fill="{ink}" transform="translate({f(ox + H + GAP)} {f(oy)}) scale({f(SC)})" d="{WORD}"/>')


# Ink bounds of the lockup, relative to (ox, baseline): the mark's box inset,
# and the word's ascender and descender (no descenders in "ember").
ASC = 0.72 * F
left = H * INSET / 100
right = H + GAP + WORD_W * SC
top = min(-MID - H / 2 + H * INSET / 100, -ASC)
bottom = max(-MID + H / 2 - H * INSET / 100, 0)


def svg(w, h, body, title):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {f(w)} {f(h)}" width="{f(w)}" height="{f(h)}" '
            f'role="img" aria-label="{title}"><title>{title}</title>{body}</svg>\n')


def write(name, text):
    with open(os.path.join(OUT, name), "w", newline="\n") as fh:
        fh.write(text)


# 1. The mark on its own, for light and for dark backgrounds.
write("ember-mark.svg", svg(100, 100, mark(BLACK), "Ember"))
write("ember-mark-dark.svg", svg(100, 100, mark(WHITE), "Ember"))

# 2. The app icon: the mark on a rounded coal tile.
tile = f'<rect x="0" y="0" width="100" height="100" rx="23" fill="{BLACK}"/>'
s = 0.66
write("ember-icon.svg", svg(100, 100, tile + mark(WHITE, 50 - 50 * s, 50 - 50 * s, s), "Ember"))

# 3. Lockups, trimmed to their ink with a margin of a tenth of the font size.
m = 0.1 * F
w, h = right - left + 2 * m, bottom - top + 2 * m
ox, oy = m - left, m - top
write("ember-lockup.svg", svg(w, h, lockup(BLACK, BLACK, ox, oy), "Ember"))
write("ember-lockup-dark.svg", svg(w, h, lockup(WHITE, WHITE, ox, oy), "Ember"))

# 4. The README banner: the dark lockup centred on a coal card.
BW, BH = 1280.0, 360.0
scale = 540.0 / (right - left)                 # the lockup is 540 wide
lw, lh = (right - left) * scale, (bottom - top) * scale
bx, by = (BW - lw) / 2 - left * scale, (BH - lh) / 2 - top * scale
body = (f'<rect width="{f(BW)}" height="{f(BH)}" rx="28" fill="{BLACK}"/>'
        f'<g transform="translate({f(bx)} {f(by)}) scale({f(scale)})">{lockup(WHITE, WHITE, 0, 0)}</g>')
write("ember-banner.svg", svg(BW, BH, body, "Ember"))
