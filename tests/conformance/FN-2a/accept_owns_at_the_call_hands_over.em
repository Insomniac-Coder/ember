#$ test: run-pass
#$ rules: FN-2a, OWN-9, OWN-11
#$ stdout: [1, 2]
#$ stdout: [1, 2]
#$ stdout: [3]
#$ stdout: [1, 2] ['ann']
# `[FN-2a]` (verdicts 147, 149) — `owns` at the call hands the argument over: `take(owns copy of xs)`
# gives a copy and keeps `xs`, `take(owns [3])` a list made in the call, `take(owns xs)` hands `xs`
# itself over; `push` takes its value the same way (`names.push(owns p.name)`).

struct Player:
    name: String
    hp: int

fn take(owns xs: Array[int]):
    println(xs)

fn main():
    xs = [1, 2]
    take(owns copy of xs)
    take(owns xs)
    take(owns [3])
    p = Player(name=String("ann"), hp=10)
    names: Array[String] = []
    names.push(owns p.name)
    println([1, 2], names)
