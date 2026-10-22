#$ test: run-pass
#$ rules: FFI-17a, FFI-44
#$ profiles: debug, development, release
#$ stdout: 42 1
#$ stdout: 42 0
#$ stdout: 2.5
#$ stdout: 2 1
# `[FFI-17a]`, `[FFI-44]` (ODR-199) — a `std::function<R(A…)>` result is, with its entry's
# `@ffi(std_function)`, an Ember `owned fn(A…) -> R`: called as any other callable, its target
# the target deleted with the value. Without the word it is a `CppFunction`, an object Ember
# owns and does not call. (`main`'s `add_two` lives to `main`'s end: one guard each line counts.)

from std.ffi import CppFunction
import cpp "functions.hpp" with (overlay = "overlays/functions.em")

fn call(f: fn(i32) -> i32, x: i32) -> i32:
    return f(x)

fn used() -> i32:
    add_two = cpp.make_adder(2).unwrap()
    return add_two(40)

fn kept_while_held() -> i32:
    kept: CppFunction = cpp.make_kept(3).unwrap()
    _ = kept.as_ptr()
    return cpp.live_guards()

fn main():
    add_two = cpp.make_adder(2).unwrap()
    println(call(add_two, 40), cpp.live_guards())
    println(used(), cpp.live_guards() - 1)
    mix = cpp.make_mixer(0.25).unwrap()
    println(mix(4.0, 2.0))
    println(kept_while_held(), cpp.live_guards())
