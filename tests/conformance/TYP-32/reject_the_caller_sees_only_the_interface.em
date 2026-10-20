#$ test: compile-fail
#$ rules: TYP-32
#$ error[E2020]: `+` cannot be applied to `some Display` and `an integer`
# `[TYP-32]` — the caller sees only `I`: the `int` that `five` returns is a `some Display` to it,
# which shows but has no `+`.

fn five() -> some Display:
    return 5

fn main():
    println(five() + 1)
