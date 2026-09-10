#$ test: compile-fail
#$ rules: BRW-4, DIA-7a
# A method call defeating disjoint-field access is B8, not B1: `[BRW-4]`'s
# parenthetical ("not through a method call — a method takes all of `self`")
# means `p.set_x(10)` takes all of `p`, so the live `ref mut p.x` conflicts
# with the call rather than with a second borrow. `[DIA-7a]` keys that shape
# to `E3025`. (Reported `E3022` before the call-provenance fix, D-040.)

struct P:
    pub x: i32
    pub y: i32
    fn set_x(mut self, v: i32):
        self.x = v

fn main():
    p = P(1, 2)
    a: ref mut i32 = ref mut p.x
    p.set_x(10)   #$ error[E3025]: cannot call a method on `p` while `p.x` is mutably borrowed
    println(a)
