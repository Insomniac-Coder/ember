#$ test: run-pass
#$ rules: MONO-8, MONO-7, MONO-9
#$ profiles: release
#$ assert-c: contains("->slot0)(x.data)")
#$ assert-c-count: contains("(ember_dyn x) {") == 1
#$ stdout: 4.0 4.0
# `[MONO-8]` — a shared body computes exactly what the specialised ones would; it costs one
# indirect call per bound-method call (the table's slot) and allocates nothing.

interface Shape:
    fn area() -> f64

struct Square:
    side: f64

extend Square implements Shape:
    fn area() -> f64:
        return this.side * this.side

struct Rect:
    w: f64
    h: f64

extend Rect implements Shape:
    fn area() -> f64:
        return this.w * this.h

@never_specialize
[T: needs Shape]
fn area_of(x: T) -> f64:
    return x.area()

fn main():
    println(area_of(Square(2.0)), area_of(Rect(2.0, 2.0)))
