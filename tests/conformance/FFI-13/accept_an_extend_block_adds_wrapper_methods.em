#$ test: run-pass
#$ rules: FFI-13
#$ profiles: debug, development, release
#$ stdout: 6 24
# `[FFI-13]` — an overlay may contain `extend` blocks with ordinary Ember wrapper methods: this is
# how an idiomatic API is layered over a raw one. `area` and `scaled` are not in the header.

import c "rect.h" with (overlay = "overlays/rect.em") as r

fn main():
    rect = r.Rect(w=2, h=3)
    println(rect.area(), rect.scaled(2).area())
