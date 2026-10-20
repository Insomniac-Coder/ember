#$ test: run-pass
#$ rules: FFI-17, FFI-3, FFI-50, FFI-24
#$ profiles: debug, development, release
#$ stdout: 11.0 12 103
#$ stdout: 5 10 3.0
#$ stdout: 3
#$ stdout: std::invalid_argument: division by zero
# `[FFI-17]` — `import cpp "engine.hpp" with (project = "engine")` parses the header as C++ in the
# configuration of the manifest's `[cpp.engine]` (`[FFI-50]`: its standard and `defines`), and
# `[FFI-3]` reaches each function through an `extern "C"` thunk the project's compiler builds,
# never through a mangled symbol. `[FFI-24]` — a `noexcept` function returns its value; one that
# may throw returns `Result[T, CppError]`, its thunk catching the exception. A `T&` parameter is
# `mut T`, a `const T&` a borrowed value; an `extern "C"` function is Part XVI's.

from std.ffi import c_int, CppError
import cpp "engine.hpp" with (project = "engine")

fn main():
    a = cpp.Vec2(x = 1.0, y = 2.0)
    b = cpp.Vec2(x = 3.0, y = 4.0)
    value: c_int = 4
    unsafe:  # SAFETY(ffi): scalar and by-value calls
        println(cpp.dot(a, b), cpp.scale(4), cpp.plain_c(3))
        cpp.bump(value)
        println(value, cpp.twice(value), cpp.sum(a))
    results: Array[Result[c_int, CppError]] = []
    unsafe:  # SAFETY(ffi): scalar calls
        results.push(cpp.checked_div(7, 2))
        results.push(cpp.checked_div(1, 0))
    for result in results:
        match result:
            Ok(quotient):
                println(quotient)
            Err(error):
                println(f"{error.type_name}: {error.what}")
