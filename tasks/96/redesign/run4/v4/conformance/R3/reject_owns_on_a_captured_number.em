#$ test: compile-fail
#$ rules: R3, R12
#$ help: write `with copy of score`
## R3, R12: in a capture list each word means what it means before a value; `owns` where the type is
## `Copy` is refused (E2085), and the message gives `copy of` (`V4/rules.md` section 3).

fn main():
    score = 1
    show = fn() with owns score: println(score)    #$ error[E2085]: `with owns score`: an `int` is copied, not handed over
    show()
