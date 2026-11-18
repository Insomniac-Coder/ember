#$ test: compile-fail
#$ rules: TYP-14
#$ error[E2072]: an `owns Mesh` reads as a `Mesh`; remove `.get()`
## `[TYP-14]` - `.get()` on an `owns` value is `E2072`.

struct Mesh:
    n: int

fn main():
    m: owns Mesh = owns Mesh(3)
    println(m.get().n)
