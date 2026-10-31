#$ test: compile-fail
#$ rules: LT-1a, GRM-41
## `[LT-1a]` — `from` names a parameter, or a view field of `this` written bare. A name that is
## neither, a field that is no view, and a function whose result is no view are `E2031`, each
## message naming `from`.

@view
struct Cursor:
    owner: ref Array[int]
    at: int

    fn first() -> ref int from at:    #$ error[E2031]: `from` names `at`, which is not a view field
        return ref this.owner[0]

    fn second() -> ref int from missing:    #$ error[E2031]: `from` names `missing`, which is not a parameter or a field of `this`
        return ref this.owner[1]

fn count(xs: ref Array[int]) -> int from xs:    #$ error[E2031]: `from` is only meaningful on a function that returns a view
    return xs.len()

fn main():
    xs = [1, 2]
    c = Cursor(ref xs, 0)
    println(c.first())
    println(c.second())
    println(count(xs))
