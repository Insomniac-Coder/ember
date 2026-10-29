#$ test: run-pass
#$ rules: FFI-44
#$ warning[L3015]: `pair_sum` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Point.manhattan` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Counter.new` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Counter.bump` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Tagged.new` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Tagged.id` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Tagged.tag` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `evens` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `largest_int` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `checked` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ warning[W5001]: macro `SQUARE` is not imported from `rows.hpp`: a function-like macro
#$ warning[W5002]: `call_with_one` is not imported from `rows.hpp`: parameter `f`
#$ warning[W5002]: `Both` is not imported from `rows.hpp`: multiple inheritance
#$ stdout: 7 7
#$ stdout: 1 2
#$ stdout: 7 3
#$ stdout: 4 6
#$ stdout: 9
#$ stdout: negative
# `[FFI-44]` — what imports automatically needs no overlay: an `extern "C"` function and POD struct,
# a standard-layout class by value with its methods, an opaque class through its thunks (one with a
# single C++ base has the base's methods too), the standard library's vector, an instantiation the
# import lists, and a function that may throw, whose exception is a `Result`.

from std.ffi import CppError, CppVector, ForeignBox, c_int
import cpp "rows.hpp" with (instantiate = ["largest<int>"])

fn pair_sum(p: cpp.Pair) -> c_int:
    unsafe:  # SAFETY(ffi): a struct of two numbers, by value
        return cpp.pair_sum(p)

fn manhattan(p: cpp.Point) -> c_int:
    unsafe:  # SAFETY(ffi): a method of a class Ember holds by value
        return p.manhattan()

fn bump_twice() -> (c_int, c_int):
    unsafe:  # SAFETY(ffi): the box owns the counter for both calls
        match cpp.Counter.new():
            Ok(counter):
                first = counter.get_mut().bump()
                return (first, counter.get_mut().bump())
            Err(error):
                panic(error.what)

fn tagged() -> Result[ForeignBox[cpp.Tagged], CppError]:
    unsafe:  # SAFETY(ffi): the constructor makes an object the box owns
        return cpp.Tagged.new()

fn id_and_tag(tagged: ref cpp.Tagged) -> (c_int, c_int):
    unsafe:  # SAFETY(ffi): the box's object, read for the calls
        return (tagged.id(), tagged.tag())

fn evens(n: c_int) -> Result[CppVector[c_int], CppError]:
    unsafe:  # SAFETY(ffi): a vector the program then owns
        return cpp.evens(n)

fn largest(a: c_int, b: c_int) -> c_int:
    unsafe:  # SAFETY(ffi): two numbers
        return cpp.largest_int(a, b)

fn checked(x: c_int) -> Result[c_int, CppError]:
    unsafe:  # SAFETY(ffi): a number
        return cpp.checked(x)

fn main():
    p = cpp.Point(x = 3, y = 4)
    println(pair_sum(cpp.Pair(a = 3, b = 4)), manhattan(p))
    (first, second) = bump_twice()
    println(first, second)
    match tagged():
        Ok(made):
            (id, tag) = id_and_tag(made.get())
            println(id, tag)
        Err(error):
            println(error.what)
    match evens(4):
        Ok(values):
            println(values.span()[2], values.span()[3])
        Err(error):
            println(error.what)
    println(largest(9, 2))
    match checked(-1):
        Ok(value):
            println(value)
        Err(error):
            println(error.what)
