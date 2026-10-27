#$ test: run-pass
#$ rules: R12, R8
#$ stdout: 1
## Verdict 40, R12 (rules.md lines 634-635, 741-742, 776-777): a list written where `score` is visible
## names it; the list never leaves `game`, so the function value provably ends first, `score` stays at
## its own address, and nothing is refused (today's E3026 is gone). AN-58's NEW program.

fn game() -> int:
    score = 0
    hs: Array[fn() changes score] = []
    hs.push(fn() changes score: score += 1)
    for ref mut h in hs:
        h()
    return score

fn main():
    println(game())
