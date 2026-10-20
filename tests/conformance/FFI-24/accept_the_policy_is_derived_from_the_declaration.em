#$ test: run-pass
#$ rules: FFI-24
#$ profiles: debug, development, release
#$ stdout: 5 3 3
#$ stdout: Odd: an odd number
#$ stdout: 2
#$ stdout: an exception that is no std::exception []
# `[FFI-24]` — every imported C++ function has an exception policy, derived from its declaration:
# `noexcept(expr)` whose `expr` holds (`quiet`) and `throw()` (`stated`) are called directly and
# return `T`; a function with no specification (`halve`, `fail`) or a `noexcept(expr)` that does
# not hold (`loud`) returns `Result[T, CppError]`, its thunk catching every exception: the
# `CppError` holds the exception's type name and `what()`, and an exception that is no
# `std::exception` is caught as well.

from std.ffi import c_int, CppError
import cpp "policies.hpp"

fn show(result: Result[c_int, CppError]) -> String:
    match result:
        Ok(value):
            return f"{value}"
        Err(error):
            return f"{error.type_name}: {error.what}"

fn main():
    direct: (c_int, c_int) = (0, 0)
    caught: Array[Result[c_int, CppError]] = []
    failed: Result[void, CppError] = Ok(())
    unsafe:  # SAFETY(ffi): scalar calls
        direct = (cpp.quiet(4), cpp.stated(0))
        caught.push(cpp.loud(1))
        caught.push(cpp.halve(3))
        caught.push(cpp.halve(4))
        failed = cpp.fail()
    println(direct.0, show(caught[0]), direct.1)
    println(show(caught[1]))
    println(show(caught[2]))
    match failed:
        Ok(_):
            println("no exception")
        Err(error):
            println(f"{error.type_name} [{error.what}]")
