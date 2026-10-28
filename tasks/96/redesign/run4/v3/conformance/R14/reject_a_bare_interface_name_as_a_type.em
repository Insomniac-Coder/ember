#$ test: compile-fail
#$ rules: R14, R3
## R3 and R14: who owns a `dyn` value is written, `owns dyn I`, `shares dyn I`, `shares mut dyn I`;
## "a bare interface name used as a type is refused, E2173" (AN-66).

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

struct Square implements Shape:
    side: int

    fn size() -> int:
        return this.side

struct Scene:
    shapes: Array[Shape]    #$ error[E2173]: say who owns the shapes: `Array[owns dyn Shape]` (the list is their only owner) or `Array[shares dyn Shape]` (other names own them too)

fn main():
    scene = Scene(shapes=[])
    println(scene.shapes.len())
