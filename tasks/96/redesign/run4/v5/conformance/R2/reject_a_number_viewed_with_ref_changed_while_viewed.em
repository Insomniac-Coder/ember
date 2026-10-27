#$ test: compile-fail
#$ rules: BRW-1
#$ help: change it after `r`'s last use, or write `r = ref mut counts[0]` and change it through `r`: `r = 9`
## design rules: R2, R8
## R2, R8 (g): `ref` before a part is needed only to view a number (`r = ref counts[0]`); the view
## then promises that the number does not change while `r` is used (E3021, with the two ways out of
## `V4/rules.md` section 3).

fn main():
    counts = [1, 2]
    r = ref counts[0]
    counts[0] = 9              #$ error[E3021]: `counts[0] = 9` (line 11) changes the number `r` reads (`r = ref counts[0]`, line 10), and `r` is used on line 12
    println(r)
