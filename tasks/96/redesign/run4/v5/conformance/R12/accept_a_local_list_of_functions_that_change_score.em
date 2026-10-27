#$ test: run-pass
#$ rules: CLO-1, BRW-16
#$ stdout: 1
## design rules: R12, R8
## R12 (rules.md lines 752-759): the list never leaves `game`, so the function value provably ends before
## `score`, which stays at its own address, and nothing is refused (today's E3026 is gone); the list
## writes nothing about what its functions change (verdict 86). AN-58's NEW program.

fn game() -> int:
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): score += 1)
    for ref mut h in hs:
        h()
    return score

fn main():
    println(game())
