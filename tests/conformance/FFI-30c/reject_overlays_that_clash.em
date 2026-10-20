#$ test: compile-fail
#$ rules: FFI-30c, FFI-30a
#$ error[E5010]: an earlier overlay has an entry for `layers_add`
#$ error[E5010]: `override` replaces an earlier overlay's entry, and none has one for `size_area`
#$ error[E5010]: an earlier overlay has an entry for `LAYERS_TWICE`
#$ error[E5010]: an earlier overlay has an entry for `Size`
#$ error[E5010]: a second entry for `layers_scale` in one overlay
#$ error[E5010]: `layers.h` declares no opaque type `Size`
#$ error[E5010]: `layers.h` declares no enum `Size`
#$ help: begin this entry with `override` to replace it
# `[FFI-30c]` (ODR-149) — a later overlay replaces an earlier one's entry only with `override`; any
# other second entry for one name is `E5010`, as is an `override` with nothing to replace, and a
# second `override` in one overlay. `[FFI-30a]` — no overlay may change a type's identity or
# layout: a `type` entry would make the struct `Size` opaque and an `enum` entry an enum (`E5010`).

import c "layers.h" with (overlay = ["overlays/theirs.em", "overlays/clash.em"])

fn main():
    pass
