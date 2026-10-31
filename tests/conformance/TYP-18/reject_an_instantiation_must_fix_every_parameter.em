#$ test: compile-fail
#$ rules: TYP-18, CLO-3
# `[TYP-18]` — an instantiation is one function only when every type
# parameter is given, including the one a callable parameter carries
# (`[CLO-3]`).

[T]
fn id(x: T) -> T:
    return x

[T]
fn twice(f: fn(T) -> T, x: T) -> T:
    return f(f(x))

[A, B]
fn pair(a: A, b: B) -> A:
    return a

fn main():
    k = pair[int]           #$ error[E2060]: `pair` has 2 type parameter(s) and `pair[…]` gives 1
    t = twice[int]          #$ error[E2060]: `twice` has 2 type parameter(s) and `twice[…]` gives 1
    q = id[int, bool]       #$ error[E2060]: `id` has 1 type parameter(s) and `id[…]` gives 2
