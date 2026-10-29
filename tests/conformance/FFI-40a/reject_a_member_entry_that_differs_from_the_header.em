#$ test: compile-fail
#$ rules: FFI-40a, FFI-12, GRM-35
#$ error[E5010]: `lookups` takes 0 parameters in `cache.hpp`, and the entry names 1
#$ error[E5010]: parameter `amount` of `add` is `count` in `cache.hpp`
#$ error[E5010]: `cache.hpp`'s `Cache` has no member `missing`
#$ error[E5010]: `cache.hpp`'s `Nothing` has no member `lookups`
# `[FFI-12]`, `[FFI-40a]` (verdict 131) — a member's entry, written under its class, matches the
# header's member: its name, and its parameters' count and names. One that differs is `E5010`, so an
# overlay cannot drift from the header.

import cpp "cache.hpp" with (overlay = "overlays/drifted.em")

fn main():
    pass
