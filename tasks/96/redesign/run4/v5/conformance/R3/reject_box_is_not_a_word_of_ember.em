#$ test: compile-fail
#$ rules: R3
#$ help: for a mesh held alone in its own block write `mesh: owns Mesh`
## R3 and the removed words of `V4/rules.md` section 3: `Box[T]` is written `owns T` (E1010; AN-40's
## message, its first way out). The nearest program that is allowed is
## `accept_owns_on_the_line_that_makes_a_value_for_an_owns_field`.

struct Mesh:
    verts: Array[int]

struct Entity:
    mesh: Box[Mesh]            #$ error[E1010]: `Box` is not part of Ember

fn main():
    println(0)
