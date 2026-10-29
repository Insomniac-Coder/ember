#$ test: run-pass
#$ rules: FFI-40a, GRM-35, UNS-7
#$ warning[L3015]: `Ledger.new` is a C++ function the program calls in unsafe code
#$ stdout: 5
# `[FFI-40a]` (ODR-160, verdict 131) — `@ffi(const)` on a member's entry, under its class, asserts
# that the method only reads its object, an asserted fact, which `ember tcb` lists: `balance` is a
# `const` method of a class with `mutable` state, and its body calls through a function pointer the
# importer cannot follow, so without the entry it would be imported as changing its object, and a
# shared borrow could not call it. `Ledger.new` has no entry, so its first call is `L3015`.

import cpp "ledger.hpp" with (overlay = "overlays/asserted.em")

fn show(ledger: cpp.Ledger):
    println(ledger.balance())

fn main():
    # SAFETY(ffi): `Ledger`'s constructor takes nothing
    unsafe:
        show(cpp.Ledger.new())
