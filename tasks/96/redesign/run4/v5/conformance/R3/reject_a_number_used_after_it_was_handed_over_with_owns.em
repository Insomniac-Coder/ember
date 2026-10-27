#$ test: compile-fail
#$ rules: R3, R1
#$ help: keep a copy instead: write `b = n` in place of line 9
## R3, R1 (verdict 84): `owns` hands over every type the same way, numbers included; after `b = owns n`
## the name `n` has ended (E3040), as it would for a list. `b = n` copies the number and keeps `n`.

fn main():
    n = 5
    b = owns n
    println(n)    #$ error[E3040]: `n` was handed over to `b` on line 9 and is used on line 10; to keep both, write `b = n` (a copy of the number) in place of line 9
