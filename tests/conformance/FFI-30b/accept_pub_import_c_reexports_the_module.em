#$ test: run-pass
#$ rules: FFI-30b
#$ profiles: debug, development, release
#$ stdout: 0 7 7
# `[FFI-30b]` — `pub import c "…" as geom` re-exports the header's module under the ordinary
# visibility rules: an importer of `vendor.gfx` reaches it as `gfx.geom`, or imports it by name.

import vendor.gfx
from vendor.gfx import geom

fn main():
    p = gfx.geom.Point(x=3, y=4)
    q: geom.Point = gfx.origin()
    unsafe:  # SAFETY(ffi): by-value calls
        println(geom.geom_sum(q), gfx.geom.geom_sum(p), geom.geom_sum(p))
