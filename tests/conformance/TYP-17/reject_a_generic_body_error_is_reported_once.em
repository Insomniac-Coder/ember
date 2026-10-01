#$ test: compile-fail
#$ rules: TYP-17, TYP-16
# D-436 — a mistake in a generic body is reported once, where the body is
# checked with its parameters opaque. Each instance repeated it when the
# message named the instance's types (`Wrap[String]` beside `Wrap[T]`),
# which the check for a repeat compared.

struct Wrap[T]:
    x: T

fn f[T](w: Wrap[T]) -> int:
    return w.nothing()    #$ error[E1010]: `Wrap[T]` has no method named `nothing`

fn main():
    println(f(Wrap(x = "s".to_string())), f(Wrap(x = 1)))
