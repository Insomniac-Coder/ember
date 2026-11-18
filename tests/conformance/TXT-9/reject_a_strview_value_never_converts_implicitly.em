#$ test: compile-fail
#$ rules: TXT-9, LEX-20
## `[TXT-9]` - a `StrView` *value* never converts implicitly to a `String`: not as a return value, a
## field, a collection element or an initialiser. A `String` of your own is `String(s)`.

struct N:
    name: String

fn mk(v: StrView) -> String:
    return v    #$ error[E2020]: expected `String`, found `StrView`

fn main():
    a = "x"
    n = N(a)    #$ error[E2020]: expected `String`, found `StrView`
    xs: Array[String] = [a]    #$ error[E2020]: expected `String`, found `StrView`
    s: String = a    #$ error[E2020]: expected `String`, found `StrView`
    println(mk(a), n.name, xs[0], s)
