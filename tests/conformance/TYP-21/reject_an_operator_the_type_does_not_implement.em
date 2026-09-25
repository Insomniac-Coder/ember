#$ test: compile-fail
#$ rules: TYP-21, TYP-40
# D-315 — a type has an operator only by implementing its interface in the
# prelude (`[TYP-40]`); a method that merely shares the name is not one. A
# `Set`'s `add` inserts, so `s + 3` is not a call of it.

struct Meters:
    v: f64

extend Meters:
    fn add(self, other: Meters) -> Meters:
        return Meters(v=self.v + other.v)

    fn neg(self) -> Meters:
        return Meters(v=-self.v)

    fn pow(self, n: int) -> Meters:
        return self

fn main():
    s: Set[int] = {1, 2}
    t = s + 3    #$ error[E2020]: `+` cannot be applied to `std.collections.Set[i64, std.collections.DefaultHasher]` and `an integer`
    m = Meters(v=1.0)
    n = m + m    #$ error[E2020]: `+` cannot be applied to `Meters`
    k = -m    #$ error[E2020]: `-` cannot be applied to `Meters`
    j = m ** 2    #$ error[E2020]: `**` takes a number, not `Meters`
    m += m    #$ error[E2020]: `+=` is not defined on `Meters`
