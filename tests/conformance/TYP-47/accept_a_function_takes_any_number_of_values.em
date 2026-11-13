#$ test: run-pass
#$ rules: TYP-47, MONO-1
#$ profiles: debug, development, release
#$ stdout: hp, 7, 2.5
#$ one
#$
#$ 3 1 0
#$ !, hp, 7
# `[TYP-47]` — `[Ts...: needs Display]` above `fn log(values: Ts...):` takes any number of values,
# each of its own type, none included; the needs apply to each of the pack's types. `for v in
# values:` is written out once per value, `values.len()` is their number, known when compiling,
# and `values...` passes them on. Each use is made for its exact types (`[MONO-1]`).

[Ts...: needs Display]
fn log(values: Ts...):
    first = true
    for v in values:
        if not first:
            print(", ", end="")
        print(v, end="")
        first = false
    println()

[Ts...]
fn count(values: Ts...) -> int:
    return values.len()

[Ts...: needs Display]
fn shout(values: Ts...):
    log("!", values...)

fn main():
    hp = 7
    log("hp", hp, 2.5)
    log("one")
    log()
    println(count(1, "two", 3.0), count(true), count())
    shout("hp", hp)
