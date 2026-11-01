#$ test: run-pass
#$ rules: IFC-1, IFC-3, TYP-20, TYP-21, TYP-24

interface Shape:
    fn area() -> f32
    # A default body: an implementer that says nothing gets this one.
    fn sides() -> i32:
        return 0

struct Square implements Shape:
    side: f32

    fn area() -> f32:
        return this.side * this.side

    fn sides() -> i32:
        return 4

struct Circle implements Shape:
    r: f32

    fn area() -> f32:
        return this.r * this.r * 3.0

struct Vec2:
    x: f32
    y: f32

# `[IFC-1]` — `extend T:` adds inherent methods from outside the type body.
extend Vec2:
    fn length_squared() -> f32:
        return this.x * this.x + this.y * this.y

# `[TYP-21]` — an operator on a non-scalar is a call of the prelude's
# operator interface (`[MOD-5]`), here `Add`.
extend Vec2 implements Add:
    type Output = Vec2

    fn add(rhs: Vec2) -> Vec2:
        return Vec2(this.x + rhs.x, this.y + rhs.y)

# A `mut self` method writes through to the caller's value.
struct Counter:
    value: i32

    fn get() -> i32:
        return this.value

    fn bump():
        this.value = this.value + 1

    fn advance(by: i32):
        this.value = this.value + by

# A `mut` parameter is an inout, not a copy.
fn double(ref mut n: i32):
    n = n + n

fn main():
    s = Square(3.0)
    println(s.area())
    println(s.sides())

    # The default body, because `Circle` does not define `sides`.
    c = Circle(2.0)
    println(c.area())
    println(c.sides())

    v = Vec2(3.0, 4.0)
    println(v.length_squared())

    total = Vec2(1.0, 2.0) + Vec2(10.0, 20.0)
    println(total.x)
    println(total.y)

    k = Counter(10)
    k.bump()
    k.bump()
    k.advance(5)
    println(k.get())

    n: i32 = 21
    double(ref mut n)
    println(n)
#$ stdout: 9.0
#$ 4
#$ 12.0
#$ 0
#$ 25.0
#$ 11.0
#$ 22.0
#$ 17
#$ 42
