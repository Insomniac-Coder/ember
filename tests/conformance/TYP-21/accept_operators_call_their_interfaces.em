#$ test: run-pass
#$ rules: TYP-21, TYP-40, IFC-4
#$ profiles: debug, development, release
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

    fn add(rhs: Vec2) -> Vec2:
        return Vec2(x=this.x + rhs.x, y=this.y + rhs.y)

    fn add_assign(rhs: Vec2):
        println("add_assign")
        this.x += rhs.x
        this.y += rhs.y

    fn sub(rhs: Vec2) -> Vec2:
        return Vec2(x=this.x - rhs.x, y=this.y - rhs.y)

    fn neg() -> Vec2:
        return Vec2(x=-this.x, y=-this.y)

    fn mul(k: f64) -> Vec2:
        return Vec2(x=this.x * k, y=this.y * k)

extend Mat2 implements Mul[Vec2]:
    type Output = Vec2

    fn mul(v: Vec2) -> Vec2:
        return Vec2(x=this.a * v.x + this.b * v.y, y=this.c * v.x + this.d * v.y)

extend Mat2 implements Mul[Mat2]:
    type Output = Mat2

    fn mul(n: Mat2) -> Mat2:
        return Mat2(
            a=this.a * n.a + this.b * n.c,
            b=this.a * n.b + this.b * n.d,
            c=this.c * n.a + this.d * n.c,
            d=this.c * n.b + this.d * n.d,
        )

# The bit operators, `~`, `<<` and `**` on a program's type.
@derive(Copy)
struct Flags:
    bits: u8

extend Flags implements BitAnd, BitOr, BitXor, Not, Shl[int], Pow[int]:
    type Output = Flags

    fn bitand(rhs: Flags) -> Flags:
        return Flags(bits=this.bits & rhs.bits)

    fn bitor(rhs: Flags) -> Flags:
        return Flags(bits=this.bits | rhs.bits)

    fn bitxor(rhs: Flags) -> Flags:
        return Flags(bits=this.bits ^ rhs.bits)

    fn not() -> Flags:
        return Flags(bits=~this.bits)

    fn shl(n: int) -> Flags:
        return Flags(bits=this.bits << n)

    fn pow(n: int) -> Flags:
        return Flags(bits=this.bits ** n)

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
