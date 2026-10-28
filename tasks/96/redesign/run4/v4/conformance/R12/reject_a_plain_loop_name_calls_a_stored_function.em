#$ test: compile-fail
#$ rules: R12, R5
#$ help: for ref mut h in hs:
## R12 (rules.md lines 766-769): calling a stored function value may change its own state, so it is not
## called through a plain loop name, which only reads (E3023). AN-58 part 6's message, word for word.

fn game() -> int:
    score = 0
    hs: Array[fn() changes score] = []
    hs.push(fn() changes score: score += 1)
    for h in hs:
        h()    #$ error[E3023]: `h` only reads the function value in `hs`, and calling a stored function value may change its own state; walk the list with `for ref mut h in hs:`
    return score

fn main():
    println(game())
