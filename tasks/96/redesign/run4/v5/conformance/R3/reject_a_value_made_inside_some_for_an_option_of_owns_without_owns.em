#$ test: compile-fail
#$ rules: OWN-10, TYP-46
#$ help: write `t = Tree(value=1, left=Some(owns Tree(value=2)))`
## design rules: R3
## R3: `owns` is written on the line that makes a value wherever the place owns it alone, also inside
## an `Option` (verdict 85): `left: Option[owns Tree]` holds its tree in a block of its own, so the
## line that makes it says so inside the `Some(...)` (E2123, `V5/rules.md` section 3's message). The
## nearest program that is allowed is `run_a_value_stored_into_an_owns_place_gets_its_own_block`.

struct Tree:
    value: int
    left: Option[owns Tree] = None

fn main():
    t = Tree(value=1, left=Some(Tree(value=2)))    #$ error[E2123]: `left` holds its tree in a block of its own; say so where it is made
    println(t.value)
