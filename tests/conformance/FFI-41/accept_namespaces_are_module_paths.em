#$ test: run-pass
#$ rules: FFI-41, FFI-17
#$ warning[L3015]: `top` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `length2` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `square` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `scale` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Clock.ticks` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ warning[W5034]: `hidden` is not imported from `space.hpp`: it is in an anonymous namespace
#$ stdout: 7 25.0 25 6.0 42
# `[FFI-41]` — namespaces become module paths: `engine::math::square` is `cpp.engine.math.square`,
# and a type of one namespace keeps its identity in another's signatures (`math::scale` takes and
# returns `engine::Vec2`). A static member function is an associated function of its class; an
# anonymous namespace's entities are skipped (`W5034`).

import cpp "space.hpp"

fn main():
    v = cpp.engine.Vec2(x = 3.0, y = 4.0)
    unsafe:  # SAFETY(ffi): calls on values
        println(cpp.top(), cpp.engine.length2(v), cpp.engine.math.square(5), cpp.engine.math.scale(v, 2.0).x, cpp.engine.Clock.ticks())
