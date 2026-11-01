#$ test: compile-fail
#$ rules: LT-1a, BRW-6
# `[LT-1a]` — a result read through a `ref mut` field points into `this` as well, so it cannot name
# the field alone, a read-only result too: the object may change the list through the field while
# the result lives.

@view
struct Values:
    owner: ref mut Array[int]
    at: int

    fn peek() -> ref int from owner:
        return ref this.owner[this.at]    #$ error[E3062]: `peek` returns a view of `this`, but its first line says `from owner`

fn main():
    ys = [1, 2]
    v = Values(ref mut ys, 0)
    println(v.peek())
