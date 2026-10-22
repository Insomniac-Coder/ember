#$ test: run-pass
#$ rules: TYP-24, BLD-13
#$ profiles: debug, development, release
#$ stdout: hello, circle 2
# D-878, `[TYP-24]` — an interface's default method is every implementation's, whichever modules
# the interface and the implementation are in and in whichever order the compiler loads them: here
# `order.named`, which declares `Named` and its default `greeting`, is loaded before
# `order.circle`, which implements it. Checking gave an implementation its defaults module by
# module, before a later module's implementations were known, so `Circle` was `E2040`, "implements
# `Named` but does not define `greeting`" (helper AB met it with `std.ffi`'s `CppError` and
# `Error`'s `source`). A check that depends on the order of the module list is no reproducible build
# (`[BLD-13]`).

import order.circle
import order.named

fn main():
    c = circle.Circle(2)
    println(c.greeting(), c.radius)
