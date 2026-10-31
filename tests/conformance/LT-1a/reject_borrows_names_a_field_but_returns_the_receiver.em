#$ test: compile-fail
#$ rules: LT-1a
# `[LT-1a]` (ODR-196) — under `@borrows(self.owner)` the result may borrow what `owner` borrows
# and nothing else of `self`: a reference to the receiver's own field is `E3062`. (`advance` changes
# its object, which it takes by address; one that only read a view's would take a copy, `E3060`.)

@view
struct Cursor:
    owner: ref Array[int]
    at: int

    fn advance() -> ref int from owner:
        this.at += 1
        return ref this.at    #$ error[E3062]: the returned view points into `this`, but its first line says `from owner`

fn main():
    xs = [1, 2]
    c = Cursor(ref xs, 0)
    println(c.advance())
