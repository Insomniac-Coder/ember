#$ test: compile-fail
#$ rules: OBJ-2, GRM-8b
# `[OBJ-2]` — a bare interface name in a type position does not say who owns the value, whatever implements the
# interface: a class does too (a parameter's type, a list's element, a field, a local, a result).

interface Shape:
    fn area() -> i32

class Tile implements Shape:
    w: i32

    fn area() -> i32:
        return this.w * this.w

class Scene:
    first: Shape                    #$ error[E2173]: `Shape` is an interface, so `Shape` does not say who owns the shape; say who owns it: write `first: owns dyn Shape` in place of line 16

fn show(s: Shape) -> i32:           #$ error[E2173]: `Shape` is an interface, so `Shape` does not say who owns the shape; say who owns it: write `s: dyn Shape` in place of line 18
    return s.area()

fn many(shapes: Array[Shape]):      #$ error[E2173]: `Shape` is an interface, so `Array[Shape]` does not say who owns the shapes; say who owns them: write `shapes: Array[owns dyn Shape]` in place of line 21 (the list is their only owner)
    pass

fn main():
    tile = Tile(2)
    println(show(ref tile))
