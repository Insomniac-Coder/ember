#$ test: run-pass
#$ rules: TYP-47, CTL-4
#$ profiles: debug, development, release
#$ stdout: 1 x 2.5
#$ first 1
#$ skipped 1 2.5
#$ all 3
#$ stopped
#$ 1 0 2 0 done
# `[TYP-47]` — `for v in values:` is written out once per value, each `v` of its own type, and
# nothing runs for the loop itself. A `continue` goes on to the next value and a `break` leaves
# them all, an `else` running when no `break` did (`[CTL-4]`); a labelled jump reaches the loop
# it names.

interface Weighed:
    fn weight() -> int

extend int implements Weighed:
    fn weight() -> int:
        return this

extend f64 implements Weighed:
    fn weight() -> int:
        return 2

extend StrView implements Weighed:
    fn weight() -> int:
        return 0

[Ts...: needs (Display, Weighed)]
fn show(values: Ts...):
    first = true
    for v in values:
        if not first:
            print(" ", end="")
        print(v, end="")
        first = false
    println()

[Ts...: needs (Display, Weighed)]
fn first_heavy(values: Ts...):
    for v in values:
        if v.weight() > 0:
            println("first", v)
            break
    else:
        println("none")

[Ts...: needs (Display, Weighed)]
fn skip_light(values: Ts...):
    print("skipped")
    for v in values:
        if v.weight() == 0:
            continue
        print("", v)
    println()

[Ts...: needs (Display, Weighed)]
fn count_all(values: Ts...):
    n = 0
    for _ in values:
        n += 1
    else:
        println("all", n)

[Ts...: needs (Display, Weighed)]
fn stop_at_light(values: Ts...):
    for v in values:
        if v.weight() == 0:
            break
    else:
        println("not stopped")
    println("stopped")

[Ts...: needs (Display, Weighed)]
fn labelled(values: Ts...):
    each: for v in values:
        for j in 0..3:
            if j == 1:
                continue each
            print(v, j, "")
    println("done")

fn main():
    show(1, "x", 2.5)
    first_heavy("x", 1, 2.5)
    skip_light("a", 1, "b", 2.5)
    count_all(1, "two", 3.0)
    stop_at_light(1, "light", 2.5)
    labelled(1, 2)
