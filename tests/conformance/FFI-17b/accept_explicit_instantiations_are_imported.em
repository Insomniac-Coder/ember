#$ test: run-pass
#$ rules: FFI-17b, FFI-17
#$ warning[L3015]: `biggest_double` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `smallest_int` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Pair_int.scale` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Pair_int.larger` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Pair_int.sum` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Pair_int.twin` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Stack_int.make` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Stack_int.push` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Stack_int.top` is a C++ function the program calls in unsafe code
#$ warning[L3015]: `Stack_int.size` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ warning[W5002]: `Stack_int.new` is not imported from `templates.hpp`: a constructor of a class template's instantiation
#$ stdout: 7 3.5 2
#$ stdout: 9
#$ stdout: 21 6 5
#$ stdout: 4 2
# `[FFI-17b]` — a template is available only as the explicit instantiations `instantiate=[…]`
# lists, each under a name made of its template-id: `geo::Pair<int>` is `geo.Pair_int`, a class of
# its template's namespace, laid out by value, with its methods; `biggest<double>` is
# `biggest_double`. Each instantiation's exception policy is derived as any function's:
# `smallest<int>` may throw, so it returns a `Result`, and so does `Pair<int>::larger`. An opaque
# one (`geo::Stack<int>`) is behind a pointer its destructor frees; Clang names no constructor of
# an instantiation to take its types from, so those are skipped (`W5002`).

from std.ffi import adopt, c_int, CppError
import cpp "templates.hpp" with (instantiate = ["geo::Pair<int>", "biggest<double>", "smallest<int>", "geo::Stack<int>"])

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
    larger = 0
    unsafe:  # SAFETY(ffi): calls on values, and on a stack the box owns
        pair.scale(3)
        match pair.larger():
            Ok(value):
                larger = value
            Err(_):
                pass
        println(pair.sum(), cpp.geo.Pair_int.twin(3).first * 2, larger - 7)
        match cpp.geo.Stack_int.make():
            Ok(made):
                stack = adopt[cpp.geo.Stack_int](made)
                _ = stack.get_mut().push(2)
                _ = stack.get_mut().push(4)
                println(stack.get().top(), stack.get().size())
            Err(error):
                println(error.what)
