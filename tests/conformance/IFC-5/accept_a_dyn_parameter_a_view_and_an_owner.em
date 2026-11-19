#$ test: run-pass
#$ rules: IFC-5, TYP-22
#$ stdout: 9
# `[IFC-5]` — `owns dyn I` holds a value of some
# type implementing `I` alone in a block of its own. (`dyn I` as a parameter's type and `ref dyn I` over a
# struct are M11's.)

interface Shape:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

struct Scene:
    main: owns dyn Shape

fn main():
    scene = Scene(owns Square(3))
    println(scene.main.area())
