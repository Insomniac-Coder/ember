#$ test: run-pass
#$ rules: FFI-17b, FFI-17
#$ profiles: debug, development, release
#$ stdout: 7 3.5 2
#$ stdout: 9
# `[FFI-17b]` — a template is available only as the explicit instantiations `instantiate=[…]`
# lists, each under a name made of its template-id: `geo::Pair<int>` is `geo.Pair_int`, a class of
# its template's namespace, laid out by value; `biggest<double>` is `biggest_double`. Each instantiation's exception
# policy is derived as any function's: `smallest<int>` may throw, so it returns a `Result`.

from std.ffi import c_int, CppError
import cpp "templates.hpp" with (instantiate = ["geo::Pair<int>", "biggest<double>", "smallest<int>"])

fn main():
    pair = cpp.geo.Pair_int(first = 3, second = 4)
    smallest: Result[c_int, CppError] = Ok(0)
    unsafe:  # SAFETY(ffi): calls on values
        println(pair.first + pair.second, cpp.biggest_double(1.5, 3.5), 2)
        smallest = cpp.smallest_int(9, 12)
    match smallest:
        Ok(value):
            println(value)
        Err(error):
            println(error.what)
