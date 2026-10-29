#$ test: run-pass
#$ rules: FFI-40a, FFI-40
#$ warning[L3015]: `Cache.new` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.lookup` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.note` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.lookups` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.hits` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.add` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.reset` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.hand_on` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.peeked` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.scaled` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.twice` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Cache.total` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 14 2 1
#$ stdout: 7 3 7 20 6
# `[FFI-40a]` (ODR-160) — `const` is no aliasing guarantee in C++: a `const` method may change
# `mutable` state. For a class with a `mutable` member the importer reads each `const` method's
# body: one that changes a field takes `mut self`, so it is called only through an exclusive
# borrow (`get_mut()`); one that changes nothing takes `self`, and a shared borrow calls it.

import cpp "cache.hpp"

fn main():
    unsafe:  # SAFETY(ffi): the box owns the cache for every call
        cache = cpp.Cache.new()
        first = cache.get_mut().lookup(3)
        second = cache.get_mut().lookup(4)
        cache.get_mut().note()
        println(first + second, cache.get().lookups(), cache.get().hits())
        cache.get_mut().add(5)
        lookups = cache.get().lookups()
        cache.get_mut().reset()
        cache.get_mut().hand_on()
        cache.get_mut().hand_on()
        cache.get_mut().hand_on()
        println(lookups, cache.get().peeked(), cache.get().scaled(2), cache.get_mut().twice(5), cache.get().total())
