#$ test: run-pass
#$ rules: FFI-32, FFI-40, FFI-41, FFI-17
#$ profiles: debug, development, release
#$ warning[W5033]: `Counter.consume` is not imported from `classes.hpp`: an `&&`-qualified member function
#$ stdout: 27 0 13 3
#$ stdout: 8 2 std::out_of_range: negative
#$ stdout: 1
# `[FFI-32]` — a standard-layout, trivially copyable class is a `@layout(c)` struct by value, with
# its constructor as `new` and its methods; any other class is opaque, made only by its
# constructor, which returns a `ForeignBox` its destructor frees. `[FFI-40]` — a `const` method is
# `self`, any other `mut self`; an `&&`-qualified one is skipped (`W5033`). `[FFI-41]` — a static
# member function is an associated function. `[FFI-17]` — each is one thunk.

from std.ffi import c_int, CppError
import cpp "classes.hpp"

fn points() -> (c_int, c_int, c_int, c_int):
    q = cpp.Point(x = 1, y = 2)
    unsafe:  # SAFETY(ffi): calls on values
        p = cpp.Point.new(3, 4)
        p.shift(10)
        return (p.sum(), cpp.Point.origin().sum(), p.x, q.sum())

fn counter() -> (c_int, c_int, String):
    unsafe:  # SAFETY(ffi): the box owns the object for every call
        match cpp.Counter.new(5):
            Ok(counter):
                _ = counter.get_mut().add(3)
                match counter.get_mut().add(-1):
                    Ok(_):
                        return (0, 0, "no exception".to_string())
                    Err(error):
                        return (counter.get().total(), counter.get().count(), f"{error.type_name}: {error.what}")
            Err(error):
                return (0, 0, error.what)

fn main():
    (a, b, c, d) = points()
    println(a, b, c, d)
    (total, count, caught) = counter()
    println(total, count, caught)
    unsafe:  # SAFETY(ffi): a static counter's address, read once
        println(cpp.Counter.destroyed().read())
