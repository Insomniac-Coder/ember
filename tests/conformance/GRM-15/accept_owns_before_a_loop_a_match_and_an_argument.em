#$ test: run-pass
#$ rules: GRM-15, GRM-27, CTL-1
#$ stdout: 3
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3
#$ stdout: 2
#$ stdout: 2
#$ stdout: 4
## `[GRM-15]` — before the iterable of a `for` and the scrutinee of a `match`, where the default is a view,
## so that the loop or the branches take the value over; before an argument for an `owns` parameter,
## whatever is given (`take(owns xs)`, `take(owns copy of xs)`, `take(owns [1, 2])`); and `owns` before a
## lambda covers the whole lambda.

fn take(owns xs: Array[int]) -> int:
    return xs.len()

fn main():
    xs = [1, 2, 3]
    println(take(owns copy of xs))
    for x in owns xs:
        println(x)
    o: Option[int] = Some(2)
    match owns o:
        Some(v):
            println(v)
        None:
            println(0)
    ys = [1, 2]
    println(take(owns ys))
    println(take(owns [1, 2, 3, 4]))
