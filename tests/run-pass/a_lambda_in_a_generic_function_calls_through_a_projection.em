#$ test: run-pass
#$ stdout: 8
#$ 6
# D-257 — a lambda in a generic body is emitted when its own types are concrete, but one calling an
# associated function through a bound (`P.O.apply(x)`, an interface method with no body) is over
# the body's opaque parameters too: emitted, it named a function never checked, and the compiler
# stopped ("every DefId names a function that was checked"). Each instance makes its own.
struct Holder:
    f: fn(int) -> int

interface Op:
    fn apply(x: int) -> int

interface HasOp:
    type O: Op

struct Twice:
    pass

extend Twice implements Op:
    fn apply(x: int) -> int:
        return x * 2

struct Pick:
    pass

extend Pick implements HasOp:
    type O = Twice

fn make[P: HasOp]() -> Holder:
    return Holder(f=owned fn(x: int) => P.O.apply(x))

fn direct[P: HasOp](x: int) -> int:
    return P.O.apply(x)

fn main():
    println(direct[Pick](4))
    h = make[Pick]()
    println(h.f(3))
