# Clause BRW-1.5: "Two names of elements of one list, at least one `ref mut`, whose indexes are not
# proved different are refused as two such arguments are (`[BRW-5]`)."
#$ test: compile-fail
#$ rules: BRW-1, BRW-5, BRW-11, BCK-3
# A `ref mut` name and a read-only one, refused at the second as `[BRW-11]` refuses such arguments
# (`E3021`, one of the two only reads). Two locals with constant values are not constants (the list
# of proofs is closed: `decisions.md` §4, area 2 Q2). Codes only.

struct Body:
    x: int

fn given(i: int, j: int):
    bodies = [Body(x=1), Body(x=2)]
    a = ref mut bodies[i]
    b = bodies[j]    #$ error[E3021]
    a.x += b.x

fn from_locals():
    bodies = [Body(x=1), Body(x=2)]
    i = 0
    j = 1
    a = ref mut bodies[i]
    b = bodies[j]    #$ error[E3021]
    a.x += b.x

fn main():
    given(0, 1)
    from_locals()
