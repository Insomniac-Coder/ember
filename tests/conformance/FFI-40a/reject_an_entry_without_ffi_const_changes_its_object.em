#$ test: compile-fail
#$ rules: FFI-40a, GRM-35
#$ warning[L3015]: `Cache.new` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.hits` is a C++ function the program calls in unsafe code
#$ help: write `@ffi(const)` above its entry, `fn lookups(…)`
# `[FFI-40a]` (verdict 132) — an overlay entry says what its method does. C++ declares `lookups`
# `const` and its body changes nothing, so with no entry it would be imported as only reading its
# object (as `hits` is); its entry has no `@ffi(const)`, so it is imported as changing it, and a
# shared borrow of the object cannot call it. The help names the line to write.

import cpp "cache.hpp" with (overlay = "overlays/unwritten.em")

fn main():
    unsafe:  # SAFETY(ffi): the box owns the cache for every call
        cache = cpp.Cache.new()
        shared = cache.get()
        println(shared.lookups())    #$ error[E3021]: cannot write through a shared reference
        println(shared.hits())
