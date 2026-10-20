#$ test: run-pass
#$ rules: FFI-11f
#$ profiles: debug, development, release
#$ warning[W5001]: macro `LIFETIMEBOUND` is not imported from `lifetimes.hpp`
#$ stdout: 5
#$ stdout: 3 10
#$ stdout: 9
# `[FFI-11f]` — a result lifetime `[[clang::lifetimebound]]` derives imports as an asserted fact:
# `pick`'s `const int&` result is a `ref` borrowing `values` (a free function's `T&` is otherwise
# a raw pointer), `larger`'s `int&` a `ref mut` borrowing both `a` and `b`, and `last_of`'s a `ref`
# borrowing the receiver and `other`. C++ does not enforce the attribute, so the fact is never
# checked: `ember tcb` lists it as asserted.

from std.ffi import CppError, CppVector, ForeignBox, c_int
import cpp "lifetimes.hpp"

fn made() -> Result[CppVector[c_int], CppError]:
    unsafe:  # SAFETY(ffi): a vector the program then owns
        return cpp.numbers()

fn second(values: CppVector[c_int]) -> ref c_int:
    unsafe:  # SAFETY(ffi): `values` is a live vector of three
        return cpp.pick(values, 1)

@borrows(a, b)
fn bigger(mut a: i32, mut b: i32) -> ref mut i32:
    unsafe:  # SAFETY(ffi): two numbers the caller lends
        return cpp.larger(a, b)

fn shelf() -> Result[ForeignBox[cpp.Shelf], CppError]:
    unsafe:  # SAFETY(ffi): the constructor makes a shelf the box owns
        return cpp.Shelf.new()

fn last(of: ref cpp.Shelf, other: ref cpp.Shelf) -> ref c_int:
    unsafe:  # SAFETY(ffi): both shelves are live
        return of.last_of(other)

fn main():
    match made():
        Ok(values):
            println(second(values))
        Err(error):
            println(error.what)
    a: i32 = 3
    b: i32 = 8
    larger = bigger(a, b)
    larger = 10
    println(a, b)
    match shelf():
        Ok(mine):
            match shelf():
                Ok(theirs):
                    println(last(mine.get(), theirs.get()))
                Err(error):
                    println(error.what)
        Err(error):
            println(error.what)
