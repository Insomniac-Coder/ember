#$ test: compile-fail
#$ rules: OWN-9
# `[OWN-9]` — an argument for an `owns` parameter is handed over with no word, numbers included:
# `take(n)` ends `n`, and a later use is `E3040`.

fn take(owns x: int):
    println(x)

fn main():
    n = 5
    take(owns n)
    println(n)  #$ error[E3040]
