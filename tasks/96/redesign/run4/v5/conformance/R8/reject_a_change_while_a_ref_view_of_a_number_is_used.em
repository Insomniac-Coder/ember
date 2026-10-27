#$ test: compile-fail
#$ rules: R8, R2
#$ help: r = ref mut counts[0]
## R8 (g): `ref` before a number part makes a read-only view of it (R2); the promise holds for it as
## for any read-only name.

fn main():
    counts = [1, 2]
    r = ref counts[0]
    counts[0] = 5    #$ error[E3021]: `counts[0] = 5` (line 10) changes the number `r` reads (`r = ref counts[0]`, line 9), and `r` is used on line 11; change it after `r`'s last use, or write `r = ref mut counts[0]` and change it through `r`: `r = 5`
    println(r)
