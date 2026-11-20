# Clause CLO-10.1: "a state of one word or less that the function's body never writes is kept in the
# state word itself, with no block and no `Alloc`"; "A function value that uses real variables and
# captured nothing with a word keeps that address in its state word, with no block and no `Alloc`";
# "A named function, or a lambda that captures nothing, has no state and allocates nothing"
#$ test: run-pass
#$ rules: CLO-10, EFF-5, COST-3
#$ stdout: 27
# Under `@noalloc`: a `copy of` a number it only reads, real variables only, a named function, and a
# copy of a number beside a copy of a function with no state (zero bytes: the state is still one
# word, as A9-6 asks the C backend to keep it).

fn seven() -> int:
    return 7

@noalloc
fn sums(k: int, ref mut total: int) -> int:
    a = fn() copy of k => k * 2
    b = fn() => total + k
    g = seven
    c = fn() copy of k, copy of g => g() + k
    return a() + b() + g() + c()

fn main():
    t = 1
    println(sums(3, ref mut t))
