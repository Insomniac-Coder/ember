#$ test: run-pass
#$ rules: FFI-24
#$ warning[L3015]: `quiet` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `stated` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `loud` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `halve` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `fail` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Gauge.new` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Gauge.read` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Gauge.strain` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `twice_int` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 5 3 3
#$ stdout: Odd: an odd number
#$ stdout: 2
#$ stdout: an exception that is no std::exception []
#$ stdout: 6 12 14
# `[FFI-24]` — every imported C++ function has an exception policy, derived from its declaration:
# `noexcept(expr)` whose `expr` holds (`quiet`) and `throw()` (`stated`) are called directly and
# return `T`; a function with no specification (`halve`, `fail`) or a `noexcept(expr)` that does
# not hold (`loud`) returns `Result[T, CppError]`, its thunk catching every exception: the
# `CppError` holds the exception's type name and `what()`, and an exception that is no
# `std::exception` is caught as well. The same holds of a constructor's and a method's
# `noexcept(expr)` (`Gauge`), and of one an instantiation evaluates (`twice<int>`).

from std.ffi import c_int, CppError
import cpp "policies.hpp" with (instantiate = ["twice<int>"])

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
    unsafe:  # SAFETY(ffi): calls on a value
        gauge = cpp.Gauge.new(6)
        println(gauge.read(), show(gauge.strain()), cpp.twice_int(7))
