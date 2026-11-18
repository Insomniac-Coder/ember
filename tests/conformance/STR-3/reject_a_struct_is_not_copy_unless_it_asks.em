#$ test: compile-fail
#$ rules: STR-3
## `[STR-3]` - `Copy` is never implicit: a struct of integers is handed over by `=` and its name is not
## used again; a struct with a `drop` is handed over too.

struct Plain:
    x: int
    y: int

struct Owner:
    id: int

    fn drop():
        println("drop")

fn main():
    p = Plain(1, 2)
    q = p
    println(p.x)    #$ error[E3040]: `p` was handed over to `q` on line 18 and is used on line 19
    o = Owner(1)
    o2 = o
    println(o.id)    #$ error[E3040]: `o` was handed over to `o2` on line 21 and is used on line 22
    println(q.x, o2.id)
