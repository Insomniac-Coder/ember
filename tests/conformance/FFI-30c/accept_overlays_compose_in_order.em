#$ test: run-pass
#$ rules: FFI-30c, FFI-30a
#$ profiles: debug, development, release
#$ stdout: 12 3 10 6
# `[FFI-30c]` (ODR-149) — `overlay` takes a list, applied first to last: the package's own overlay,
# listed after its dependency's, replaces `layers_scale`'s entry with `override` (it is now safe to
# call), and adds a `rename` and an entry the first did not state. `[FFI-30a]` — no overlay changes
# a type's identity: `Extent`, the first overlay's name for `Size`, is `Size`.

import c "layers.h" with (overlay = ["overlays/theirs.em", "overlays/ours.em"])

fn main():
    extent: c.Extent = c.Size(w = 2, h = 3)
    println(c.layers_scale(3, 4), c.add(1, 2), c.LAYERS_TWICE(5), c.size_area(extent))
