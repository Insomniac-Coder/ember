#$ test: compile-fail
#$ rules: LT-1a
# `[LT-1a]` (ODR-196) — `-> T from f` names a view field of the object written bare: a field whose
# type is a reference or view. A field that is not one, and a name that is neither a parameter nor a
# field of `this` (a field the type does not have, one of an enum, one with no object), are `E2031`.

@view
struct Cursor:
    owner: ref Array[int]
    at: int

    #$ error[E2031]: `from` names `at`, which is not a view field
    fn first() -> ref int from at:
        return ref this.owner[0]

    #$ error[E2031]: `from` names `missing`, which is not a parameter or a field of `this`
    fn second() -> ref int from missing:
        return ref this.owner[1]

@view
[T]
struct Each:
    owner: ref Array[T]
    at: int

    #$ error[E2031]: the view fields of `Each` are `owner`
    fn first() -> ref T from at:
        return ref this.owner[0]

enum Pick:
    A
    B

    #$ error[E2031]: `from` names `name`, which is not a parameter or a field of `this`
    fn label() -> StrView from name:
        return "a"

#$ error[E2031]: `from` names `owner`, which is not a parameter or a field of `this`
fn free(xs: ref Array[int]) -> ref int from owner:
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
