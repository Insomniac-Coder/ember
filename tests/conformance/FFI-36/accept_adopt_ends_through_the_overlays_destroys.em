#$ test: run-pass
#$ rules: FFI-36, FFI-36a
#$ warning[W5002]: `Enemy.new` is not imported from `enemies.hpp`: a constructor of a class whose destructor is not public
#$ warning[L3015]: `spawn` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Enemy.hp` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `destroyed_count` is a C++ function the program calls in unsafe code
#$ stdout: 4 1
# `[FFI-36]` — `ffi.adopt[T](p)` ends what it takes through `T`'s one destructor: where the class's
# overlay entry writes `@ffi(destroys(f))`, that `f`, here `destroy_enemy` for a class whose own
# destructor is not public. It runs once, when the `owns Enemy` ends.

from std.ffi import adopt
import cpp "enemies.hpp" with (overlay = "overlays/enemies.em")

fn hit_points() -> i32:
    # SAFETY(ffi): `spawn` gives a new object nothing else owns
    unsafe:
        enemy = adopt[cpp.Enemy](cpp.spawn(4))
        return enemy.hp()

fn main():
    hp = hit_points()
    # SAFETY(ffi): a counter's read
    unsafe:
        println(hp, cpp.destroyed_count())
