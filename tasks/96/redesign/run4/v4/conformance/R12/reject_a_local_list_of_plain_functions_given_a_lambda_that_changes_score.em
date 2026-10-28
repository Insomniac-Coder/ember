#$ test: compile-fail
#$ rules: R12
#$ help: hs: Array[fn() changes score] = []
## Verdicts 22 and 40, R12 (rules.md lines 634-635, 645-649): a plain `fn()` changes nothing, so a list
## of `fn()` does not take a function that changes `score` (E2113); where `score` is visible the list
## names it. AN-58 part 6's message, word for word.

fn game() -> int:
    score = 0
    hs: Array[fn()] = []
    hs.push(fn() changes score: score += 1)    #$ error[E2113]: this function changes `score`, a variable it captured (`fn() changes score`), and `hs` holds `fn()`, functions that change nothing; keep it in a list that says so: `hs: Array[fn() changes score] = []`
    for ref mut h in hs:
        h()
    return score

fn main():
    println(game())
