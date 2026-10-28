#$ test: compile-fail
#$ rules: R3
#$ help: write `h: owns Handle`
## R3 and the removed words of `V4/rules.md` section 3: `ForeignBox[T]` is written `owns T` (E1010; the
## `ForeignBox` row of `V4/vocabulary.md`; completeness gap G8). The nearest program that is allowed is
## `accept_owns_on_the_line_that_makes_a_value_for_an_owns_field`.

struct Handle:
    n: int

struct Holder:
    h: ForeignBox[Handle]      #$ error[E1010]: `ForeignBox` is not part of Ember

fn main():
    println(0)
