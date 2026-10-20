#$ test: compile-fail
#$ rules: MONO-5, MONO-7
## `[MONO-7]` — `@never_specialize` requires one shared body, which a generic using its type
## parameter other than as the receiver of its bounds' methods cannot have: `E2223`, at the use.

interface Scale:
    fn factor(self) -> f64

struct Unit:
    f: f64

extend Unit implements Scale:
    fn factor(self) -> f64:
        return self.f

@never_specialize
fn twice[T: Scale](x: T) -> f64:
    y: T = x  #$ error[E2223]: `twice` cannot share one body: `T` is named here
    return y.factor() * 2.0

@never_specialize
fn first[T: Scale](xs: Array[T]) -> f64:  #$ error[E2223]: `first` cannot share one body: `xs`'s type uses a type parameter other than as the whole type of a borrowed parameter
    return xs[0].factor()

fn main():
    println(twice(Unit(1.0)), first([Unit(2.0)]))
