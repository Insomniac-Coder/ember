#$ test: run-pass
#$ rules: TYP-21, TYP-40, IFC-4
#$ profiles: debug, release, shipping
#$ stdout: add_assign
#$ 1.5 2.5 -2.5 -3.5
#$ 3.0 5.0
#$ 5.0 3.0 1.0 0.0
#$ 8 14 6 243 48 9
#$ 14
# `[TYP-21]` — an operator on a type that is not a number calls its operator
# interface. `m * v` and `m * m` are `Mul[Vec2]` and `Mul[Mat2]`, two
# instances of one interface (D-313). `a += b` calls `add_assign` where the
# type has it, and is `a = a * b` where it has only `Mul`.

@derive(Copy)
struct Vec2:
    x: f64
    y: f64

@derive(Copy)
struct Mat2:
    a: f64
    b: f64
    c: f64
    d: f64

extend Vec2 implements Add, AddAssign, Sub, Neg, Mul[f64]:
    type Output = Vec2

    fn add(self, rhs: Vec2) -> Vec2:
        return Vec2(x=self.x + rhs.x, y=self.y + rhs.y)

    fn add_assign(mut self, rhs: Vec2):
        println("add_assign")
        self.x += rhs.x
        self.y += rhs.y

    fn sub(self, rhs: Vec2) -> Vec2:
        return Vec2(x=self.x - rhs.x, y=self.y - rhs.y)

    fn neg(self) -> Vec2:
        return Vec2(x=-self.x, y=-self.y)

    fn mul(self, k: f64) -> Vec2:
        return Vec2(x=self.x * k, y=self.y * k)

extend Mat2 implements Mul[Vec2]:
    type Output = Vec2

    fn mul(self, v: Vec2) -> Vec2:
        return Vec2(x=self.a * v.x + self.b * v.y, y=self.c * v.x + self.d * v.y)

extend Mat2 implements Mul[Mat2]:
    type Output = Mat2

    fn mul(self, n: Mat2) -> Mat2:
        return Mat2(
            a=self.a * n.a + self.b * n.c,
            b=self.a * n.b + self.b * n.d,
            c=self.c * n.a + self.d * n.c,
            d=self.c * n.b + self.d * n.d,
        )

# The bit operators, `~`, `<<` and `**` on a program's type.
@derive(Copy)
struct Flags:
    bits: u8

extend Flags implements BitAnd, BitOr, BitXor, Not, Shl[int], Pow[int]:
    type Output = Flags

    fn bitand(self, rhs: Flags) -> Flags:
        return Flags(bits=self.bits & rhs.bits)

    fn bitor(self, rhs: Flags) -> Flags:
        return Flags(bits=self.bits | rhs.bits)

    fn bitxor(self, rhs: Flags) -> Flags:
        return Flags(bits=self.bits ^ rhs.bits)

    fn not(self) -> Flags:
        return Flags(bits=~self.bits)

    fn shl(self, n: int) -> Flags:
        return Flags(bits=self.bits << n)

    fn pow(self, n: int) -> Flags:
        return Flags(bits=self.bits ** n)

fn main():
    v = Vec2(x=1.0, y=2.0)
    v += Vec2(x=0.5, y=0.5)
    w = -v - Vec2(x=1.0, y=1.0)
    println(v.x, v.y, w.x, w.y)
    v *= 2.0
    println(v.x, v.y)
    m = Mat2(a=0.0, b=1.0, c=1.0, d=0.0)
    r = m * v
    mm = m * m
    println(r.x, r.y, mm.a, mm.b)
    f = Flags(bits=0b1100)
    g = Flags(bits=0b1010)
    println((f & g).bits, (f | g).bits, (f ^ g).bits, (~f).bits, (f << 2).bits, (Flags(bits=3) ** 2).bits)
    f |= g
    println(f.bits)
