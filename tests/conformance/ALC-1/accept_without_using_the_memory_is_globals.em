#$ test: run-pass
#$ rules: ALC-1
#$ stdout: 3 6
# `[ALC-1]` — a place with no `using` after its type takes its memory from the built-in manager, `Global`,
# which is a manager with no fields: it needs no instance on the line that makes the value. (`using A`
# after a type, which names another manager, is not built by this compiler yet.)

fn total(xs: Array[int]) -> int:
    sum = 0
    for x in xs:
        sum += x
    return sum

fn main():
    xs: Array[int] = [1, 2, 3]
    println(xs.len(), total(xs))
