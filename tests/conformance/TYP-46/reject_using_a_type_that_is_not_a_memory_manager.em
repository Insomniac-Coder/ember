#$ test: compile-fail
#$ rules: TYP-46
## `[TYP-46]` — `using A` names a type implementing `Allocator` (`[ALC-2]`); another is `E2040`, and
## an instance of one is `E2020`.

struct Plain:
    n: int

struct Mesh:
    points: Array[float]

struct Holder:
    mesh: owns Mesh using Plain    #$ error[E2040]: `Plain` is not a memory manager: it does not implement `std.mem.Allocator`

fn main():
    p = Plain(1)
    m = owns Mesh(points=[0.0]) using p    #$ error[E2020]: `p` is not an instance of a memory manager: `Plain` does not implement `std.mem.Allocator`
    println(m.points.len())
