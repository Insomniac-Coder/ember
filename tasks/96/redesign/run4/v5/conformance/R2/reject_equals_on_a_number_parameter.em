#$ test: compile-fail
#$ rules: R2
#$ help: for a number of your own write `count = n` and change `count`
## R2: a parameter stands for what the caller gave and keeps it for its whole scope: `=` on one is
## refused (E3023), unless it is `ref mut` (`V4/rules.md` section 3's message).

fn bump(n: int) -> int:
    n = n + 1                  #$ error[E3023]: `n` is the number the caller passed
    return n

fn main():
    println(bump(1))
