#$ test: compile-fail
#$ rules: FFI-48
#$ warning[L3015]: `Holder.new` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `ready` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `plain_sum` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Holder.plain` is a C++ function the program calls in unsafe code
#$ warning[W5002]: `fetch` is not imported from `unsupported.hpp`: a C++ coroutine (`Task` declares `promise_type`)
#$ warning[W5002]: `Holder.value` is not imported from `unsupported.hpp`: overloads differing only in their return type
#$ warning[W5002]: `origin_fixed` is not imported from `nttp.hpp`: its result: `Fixed` by value, which is opaque
#$ warning[W5002]: `Fixed` is imported from `nttp.hpp` only as an opaque type, usable behind a pointer: `Fixed<Point{1, 2}>`: `Fixed` has a non-type template parameter of class type (`Point P`)
# `[FFI-48]` — the unsupported rows, each `E5034` naming its row where the program names it: a
# coroutine (its result's class declares `promise_type`), a template with a non-type parameter of
# class type, and a cast across a C++ hierarchy (`cast … if` between classes Ember holds as C++ objects).
# Of two methods that differ only in their result (`int& value()`, `const int& value() const`), the
# first is imported and the import says why the other is not.

import cpp "unsupported.hpp"
import cpp "nttp.hpp" with (project = "nttp") as n

fn narrow(shape: ref cpp.Shape) -> i32:
    square = cast shape if cpp.Square    #$ error[E5034]: a cast across a C++ hierarchy
    return 0

fn main():
    unsafe:  # SAFETY(ffi): never run
        task = cpp.fetch(1)    #$ error[E5034]: a C++ coroutine
        fixed = n.origin_fixed()    #$ error[E5034]: a non-type template parameter of class type
        holder = cpp.Holder.new()
        println(cpp.ready(), n.plain_sum(1, 2), holder.plain())
