#$ test: run-pass
#$ rules: FFI-40a, FFI-40
#$ profiles: debug, development, release
#$ stdout: 14 2
# `[FFI-40a]` — `const` is no aliasing guarantee in C++: a `const` method may change `mutable`
# state. For a class with a `mutable` member the importer assumes every `const` method does, and
# gives each `mut self`, so it is called only through an exclusive borrow.

import cpp "cache.hpp"

fn main():
    unsafe:  # SAFETY(ffi): the box owns the cache for every call
        cache = cpp.Cache.new()
        first = cache.get_mut().lookup(3)
        second = cache.get_mut().lookup(4)
        println(first + second, cache.get_mut().lookups())
