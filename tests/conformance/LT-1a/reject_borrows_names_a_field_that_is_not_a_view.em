#$ test: compile-fail
#$ rules: LT-1a
# `[LT-1a]` (ODR-196) — `@borrows(self.f)` names a view field of the receiver: a field whose type
# is a reference or view. A field that is not one, a field the type does not have, a receiver
# that is not a struct, and a function with no receiver are `E2031`.

@view
struct Cursor:
    owner: ref Array[int]
    at: int

    @borrows(self.at)    #$ error[E2031]: `@borrows` names `self.at`, which is not a view field
    fn first(mut self) -> ref int:
        return ref self.owner[0]

    @borrows(self.missing)    #$ error[E2031]: `Cursor` has no field `missing`
    fn second(mut self) -> ref int:
        return ref self.owner[1]

@view
struct Each[T]:
    owner: ref Array[T]
    at: int

    @borrows(self.at)    #$ error[E2031]: the view fields of `Each` are `self.owner`
    fn first(mut self) -> ref T:
        return ref self.owner[0]

enum Pick:
    A
    B

    @borrows(self.name)    #$ error[E2031]: `Pick` is not a struct
    fn label(self) -> str:
        return "a"

@borrows(self.owner)    #$ error[E2031]: but this function has no receiver
fn free(xs: ref Array[int]) -> ref int:
    return ref xs[0]

fn main():
    xs = [1, 2]
    c = Cursor(ref xs, 0)
    println(c.first())
    println(c.second())
    e = Each(ref xs, 0)
    println(e.first())
    println(Pick.A.label())
    println(free(xs))
