#$ test: run-fail
#$ rules: FFI-24b
#$ panics: `risky` threw `std::runtime_error` (negative), though its overlay asserts it `noexcept`
#$ profiles: debug, development, release
# `[FFI-24b]` — the thunk of a function an overlay asserts `noexcept` still catches, and a throw
# then panics naming the declaration: never an unexplained `std::terminate`.

import cpp "policies.hpp" with (overlay = "overlays/policies.em")

fn main():
    println(cpp.risky(-1))
