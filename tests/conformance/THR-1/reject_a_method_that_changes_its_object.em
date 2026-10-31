#$ test: compile-fail
#$ rules: THR-1, EXC-15
#$ profiles: debug
#$ error[E7003]: `@sync` class `Token` cannot declare a method that changes its object: `bump` does
# `[THR-1]` (0.9.10's `@sync`, until M1 refuses it) — a `@sync` class's method may not change its
# object. What a method does to it is worked out from its body: `bump` lends it to `lend` as
# `ref mut`, so it may change it, and is refused. (A field written in a method is refused on its
# own, `E7003` too.)

@sync
class Token:
    value: int = 1

    fn bump():
        lend(ref mut this)

fn lend(ref mut t: Token):
    pass

fn main():
    t = Token()
    t.bump()
