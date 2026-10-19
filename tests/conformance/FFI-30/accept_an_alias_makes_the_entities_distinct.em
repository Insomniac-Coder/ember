#$ test: run-pass
#$ rules: FFI-30
#$ profiles: debug, development, release
#$ stdout: 5.0 3.0
# `[FFI-30]` — the help of `E5011`: an import with its own alias has its own entities, so one C
# declaration imported with two layouts is two Ember types, each with its own fields.

import helpers.narrow
import c "common.h" with (defines = ["COMMON_WIDE"]) as wide

fn main():
    v = wide.Vec2(x=1.0, y=1.0, z=1.0)
    println(narrow.length2(1.0, 2.0), v.x + v.y + v.z)
