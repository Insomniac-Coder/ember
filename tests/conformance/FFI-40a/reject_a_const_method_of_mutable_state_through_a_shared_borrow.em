#$ test: compile-fail
#$ rules: FFI-40a
# `[FFI-40a]` — the `const` methods of a class with a `mutable` member take `mut self`: a shared
# borrow of the object cannot call one.

import cpp "cache.hpp"

fn main():
    unsafe:  # SAFETY(ffi): the box owns the cache for every call
        cache = cpp.Cache.new()
        println(cache.get().lookups())    #$ error[E3021]: cannot write through a shared reference
