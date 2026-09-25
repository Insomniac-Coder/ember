#$ test: run-pass
#$ rules: LEX-15, TYP-21
#$ profiles: debug, release, shipping
#$ stdout: -6 250 false true
# ODR-040 — `not` is `Not`'s method: a method name after `fn` and after `.`,
# and the logical operator everywhere else.

@derive(Copy)
struct Bits:
    v: u8

extend Bits implements Not:
    type Output = Bits

    fn not(self) -> Bits:
        return Bits(v=~self.v)

fn main():
    b = Bits(v=5)
    println(5.not(), b.not().v, not true, not (1 > 2))
