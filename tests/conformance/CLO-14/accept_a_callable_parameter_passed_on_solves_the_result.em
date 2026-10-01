#$ test: run-pass
#$ rules: CLO-14, TYP-18, STD-19
#$ stdout: 6 12 6
# D-424 — a parameter bounded by a callable type is called as its bound says,
# so passing it on to `map` solves the adapter's `R` from that bound, as a
# function or a lambda does. `relay` was `E2060` ("cannot tell what `R` is"),
# in the explicit form and in the parameter form, and through a second
# generic function.

fn relay[G: fn(int) -> int](g: G) -> int:
    return (0..3).map(g).sum()

fn relay_form(g: fn(int) -> int) -> int:
    return (0..4).map(g).sum()

fn pass_on[F: fn(int) -> int](f: F) -> int:
    return relay(f)

fn double(x: int) -> int:
    return x * 2

fn main():
    println(relay(double), relay_form(double), pass_on(fn(v) => v + 1))
