#$ test: compile-fail
#$ rules: OWN-10, TYP-46
#$ help: write `scene.shapes.push(owns Square(side=3))`
## design rules: R3, R14
## R3, R14: `owns` is written on the line that makes a value wherever the place owns it alone, also a
## list whose values each live in their own block (verdict 85): `shapes: Array[owns dyn Shape]` is each
## shape's one owner, so the line that makes a shape for it says so, at a `push` too (E2123,
## `V5/rules.md` section 3's message in this program's names). The nearest program that is allowed is
## `accept_a_value_pushed_into_a_list_of_owns_values_says_owns`.

interface Shape:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

struct Scene:
    shapes: Array[owns dyn Shape]

fn main():
    scene = Scene(shapes=[owns Square(side=2)])
    scene.shapes.push(Square(side=3))      #$ error[E2123]: `shapes` holds each shape in a block of its own; say so where it is made
    println(scene.shapes.len())
