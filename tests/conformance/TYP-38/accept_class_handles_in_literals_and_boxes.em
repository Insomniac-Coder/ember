#$ test: run-pass
#$ rules: TYP-38, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 1 2 2
#$ true
#$ 1
# D-200 — a list literal and `Shared` own their payload. A class handle placed
# in one is retained, and each owner releases it once; the literal's elements
# move into the new buffer. (A class object always has a block of its own, so
# `owns` before a class type is `E2114`, `[TYP-45]`.)

class Token:
    id: int

fn main():
    token = Token(id=1)
    xs = [token, token]
    ys = [Token(id=2)]
    println(xs[1].id, ys[0].id, len(xs))
    _shared = Shared(token)
    println(xs[0] is token)
    println(token.id)
