#$ test: compile-fail
#$ rules: TYP-34
## `[TYP-34]` - an enum with a view payload, and a struct holding one, are view types: each lives no longer than
## what its view fields point into.

enum Pick:
    Name(StrView)
    Nothing

struct Holder:
    p: Pick

fn leak() -> Pick:
    t = String("hello")
    return Pick.Name(t)              #$ error[E3063]: this points into `t`

fn leak_nested() -> Holder:
    t = String("hello")
    return Holder(Pick.Name(t))      #$ error[E3063]: this points into `t`

fn main():
    println(1)
