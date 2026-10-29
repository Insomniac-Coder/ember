#$ test: compile-fail
#$ rules: FFI-40a
#$ warning[L3015]: `Cache.new` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.lookup` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.reset` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.add` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.note` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.hand_on` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.twice` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.lookups` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.hits` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.peeked` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.scaled` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.total` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Watcher.new` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Watcher.look` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Watcher.count` is a C++ function the program calls in unsafe code
# `[FFI-40a]` (ODR-160) — a `const` method whose body changes the `mutable` state takes `mut self`,
# whichever way the body changes it, and a shared borrow of the object cannot call it; a `mutable`
# member a base declares is the derived class's state too (`Watcher.look`).

import cpp "cache.hpp"

fn main():
    unsafe:  # SAFETY(ffi): the box owns the cache for every call
        cache = cpp.Cache.new()
        shared = cache.get()
        println(shared.lookup(3))    #$ error[E3021]: cannot write through a shared reference
        shared.reset()    #$ error[E3021]: cannot write through a shared reference
        shared.add(1)    #$ error[E3021]: cannot write through a shared reference
        shared.note()    #$ error[E3021]: cannot write through a shared reference
        shared.hand_on()    #$ error[E3021]: cannot write through a shared reference
        println(shared.twice(1))    #$ error[E3021]: cannot write through a shared reference
        println(shared.lookups(), shared.hits(), shared.peeked(), shared.scaled(2), shared.total())
        watcher = cpp.Watcher.new()
        println(watcher.get().look())    #$ error[E3021]: cannot write through a shared reference
        println(watcher.get().count())
