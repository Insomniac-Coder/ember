# Ember's logo

<p align="center"><img src="ember-lockup.svg" alt="Ember" width="420"></p>

**The Spark.** A diamond split across its horizontal diagonal: the ember in red above, the coal
below, and one spark leaving the fire. Red is always the fire. The coal is black on light
backgrounds and ash white on dark ones.

## Files

| File | Use |
|---|---|
| [`ember-mark.svg`](ember-mark.svg) | The mark on light backgrounds |
| [`ember-mark-dark.svg`](ember-mark-dark.svg) | The mark on dark backgrounds |
| [`ember-icon.svg`](ember-icon.svg) | The icon: the mark on a rounded coal tile; PNGs of it in [`png/`](png/) at 16, 32, 64, 180, 256 and 512 px |
| [`ember-lockup.svg`](ember-lockup.svg) | Mark and wordmark, on light backgrounds |
| [`ember-lockup-dark.svg`](ember-lockup-dark.svg) | Mark and wordmark, on dark backgrounds |
| [`ember-banner.svg`](ember-banner.svg) | The README banner |

All of them are plain SVG paths; the wordmark is outlined, so no font is needed to show them.

## Colours

| | Hex | Role |
|---|---|---|
| Ember red | `#E0241B` | The ember and the spark |
| Coal | `#0F0F10` | The coal on light backgrounds, the wordmark, the icon's tile |
| Ash white | `#FAFAF7` | The coal and the wordmark on dark backgrounds |

## Construction

In a 100-unit square: the diamond is centred, with a half-diagonal of 42 and corners rounded by 5.
The seam between ember and coal is 4.5 wide. The spark has a half-diagonal of 7.5, the same
corner proportion, and sits on the diamond's up-right diagonal 9 clear of its edge (twice the
seam), inside the diamond's bounding square. The icon sets the mark at 66% on a tile with corners
of 23.

The wordmark is lowercase `ember` in Space Grotesk Bold, tracked −0.045 em. Space Grotesk is
under the SIL Open Font License 1.1. In the lockup the mark's box is 1.16 times the font size,
centred 0.31 font sizes above the baseline, with 0.268 font sizes between the box and the word.

[`build.py`](build.py) writes every SVG here from these numbers (not part of the build; its
docstring says how to run it).
