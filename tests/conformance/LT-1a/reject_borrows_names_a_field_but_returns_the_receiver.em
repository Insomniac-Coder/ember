#$ test: compile-fail
#$ rules: LT-1a
# `[LT-1a]` (ODR-196) — under `@borrows(self.owner)` the result may borrow what `owner` borrows
# and nothing else of `self`: a reference to the receiver's own field is `E3062`.

@view
struct Cursor:
    owner: ref Array[int]
    at: int

    @borrows(self.owner)
    fn position(mut self) -> ref int:
        return ref self.at    #$ error[E3062]: the returned view points into `self`, and `@borrows` names only what `self.owner` borrows

fn main():
    xs = [1, 2]
    c = Cursor(ref xs, 0)
    println(c.position())
