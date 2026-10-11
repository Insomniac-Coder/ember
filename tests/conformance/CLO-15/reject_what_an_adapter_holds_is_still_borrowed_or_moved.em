#$ test: compile-fail
#$ rules: CLO-15, CLO-2, CLO-4, STD-19, FN-1
#$ profiles: debug
# `[CLO-15]` (ODR-100) — what the adapters hold keeps its rules. A lambda that
# writes a variable holds a mutable borrow of it while the iterator lives; a
# function cannot hand back a second copy of a closure that writes its captures
# (D-422: it duplicated the mutable borrow, which `other = bump` refuses); and a
# named iterator given to `zip` is moved, not copied.

fn steal[F: fn(int)](f: F) -> F:
    return f  #$ error[E3013]: cannot move out of a borrowed parameter

fn keep[F: fn(int)](owned f: F) -> F:
    return f

fn main():
    seen: Array[int] = []
    g = keep(fn(v: int) => seen.push(v))
    seen.clear()  #$ error[E3022]: `seen` is already mutably borrowed
    g(1)
    n = 0
    bump = fn(v: int):
        n += v
    other = steal(bump)
    bump(1)
    other(2)
    xs = [1, 2, 3]
    ys = [4, 5, 6]
    writer = ys.iter_mut()
    for a, b in xs.iter().zip(writer):
        b += a
    writer.next()  #$ error[E3040]: `writer` has been moved out of
