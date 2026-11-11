#$ test: compile-fail
#$ rules: FFI-13
#$ error[E1010]: cannot find field `width`
#$ profiles: debug
# `[FFI-13]` — a wrapper in an overlay's `extend` block is ordinary Ember: its body is checked as
# any method's is. `Rect` has no field `width`.

import c "rect.h" with (overlay = "overlays/broken.em") as r

fn main():
    rect = r.Rect(w=2, h=3)
    println(rect.area())
