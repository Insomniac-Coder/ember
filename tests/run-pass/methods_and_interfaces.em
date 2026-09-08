#$ test: run-pass
#$ rules: IFC-1, IFC-3, TYP-20, TYP-21, TYP-24

interface Shape:
    fn area(self) -> f32
    # A default body: an implementer that says nothing gets this one.
    fn sides(self) -> i32:
        return 0

interface Add:
    fn add(self, rhs: Vec2) -> Vec2

struct Square implements Shape:
    side: f32

    fn area(self) -> f32:
        return self.side * self.side

    fn sides(self) -> i32:
        return 4

struct Circle implements Shape:
    r: f32

    fn area(self) -> f32:
        return self.r * self.r * 3.0

struct Vec2:
    x: f32
    y: f32

# `[IFC-1]` — `extend T:` adds inherent methods from outside the type body.
extend Vec2:
    fn length_squared(self) -> f32:
        return self.x * self.x + self.y * self.y

# `[TYP-21]` — an operator on a non-scalar is an interface call.
extend Vec2 implements Add:
    fn add(self, rhs: Vec2) -> Vec2:
        return Vec2(self.x + rhs.x, self.y + rhs.y)

# A `mut self` method writes through to the caller's value.
struct Counter:
    value: i32

    fn get(self) -> i32:
        return self.value

    fn bump(mut self):
        self.value = self.value + 1

    fn advance(mut self, by: i32):
        self.value = self.value + by

# A `mut` parameter is an inout, not a copy.
fn double(mut n: i32):
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

    n = 21
    double(n)
    println(n)
#$ stdout: 9
#$ 4
#$ 12
#$ 0
#$ 25
#$ 11
#$ 22
#$ 17
#$ 42
