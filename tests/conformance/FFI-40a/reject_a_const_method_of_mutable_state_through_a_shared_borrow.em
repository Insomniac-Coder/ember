#$ test: compile-fail
#$ rules: FFI-40a
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
