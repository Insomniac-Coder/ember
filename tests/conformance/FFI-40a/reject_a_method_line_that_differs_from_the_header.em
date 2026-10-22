#$ test: compile-fail
#$ rules: FFI-40a, FFI-12
#$ error[E5010]: `Cache.lookups` is `fn lookups(self) -> c_int` in `cache.hpp`
#$ error[E5010]: `Cache.add` is `fn add(self, count: c_int)` in `cache.hpp`
#$ error[E5010]: `cache.hpp` declares no method `Cache.missing`
#$ error[E5010]: the line for `Cache.hits` takes `self` or `mut self` first
#$ error[E5010]: `cache.hpp` declares no method `Nothing.lookups`
# `[FFI-40a]` (ODR-160), `[FFI-12]` — a method line restates the header's method: its name, its
# parameters and its result as the import gives them, after `self` or `mut self`. One that differs
# is `E5010`, so an overlay cannot drift from the header.

import cpp "cache.hpp" with (overlay = "overlays/drifted.em")

fn main():
    pass
