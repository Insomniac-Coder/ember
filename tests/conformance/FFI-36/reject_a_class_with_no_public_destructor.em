#$ test: compile-fail
#$ rules: FFI-36
#$ warning[L3015]: `spawn` is a C++ function the program calls in unsafe code
#$ warning[W5002]: `Enemy.new` is not imported from `enemies.hpp`: a constructor of a class whose destructor is not public
# `[FFI-36]` — `adopt` ends what it takes through its type's one destructor; an imported C++ class
# with no public destructor has none Ember can call, so adopting one is `E5002`, whose message names
# the function of the header that ends one, for the overlay to name.

from std.ffi import adopt
import cpp "enemies.hpp"

fn main():
    # SAFETY(ffi): `spawn` gives a new object nothing else owns
    unsafe:
        enemy = adopt[cpp.Enemy](cpp.spawn(4))    #$ error[E5002]: `Enemy` has no public destructor, so nothing Ember can call ends an adopted one; name the function that does in the overlay: `@ffi(destroys(destroy_enemy))` above `type Enemy`
        println(enemy.hp())
