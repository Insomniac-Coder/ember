#$ test: compile-fail
#$ rules: FFI-40a, GRM-35
# `[FFI-40a]` (ODR-160) — an overlay's `class` section restates a method as an Ember method line,
# whose receiver wins over the importer's reading of the body (an asserted fact, which `ember tcb`
# lists): `lookups`, which changes nothing, is `mut self` here, and a shared borrow cannot call it.

import cpp "cache.hpp" with (overlay = "overlays/stricter.em")

fn main():
    unsafe:  # SAFETY(ffi): the box owns the cache for every call
        cache = cpp.Cache.new()
        println(cache.get().lookups())    #$ error[E3021]: cannot write through a shared reference
        println(cache.get().hits(), cache.get_mut().lookups())
