#$ test: compile-fail
#$ rules: R12, R5
#$ help: show = fn(): println(score)
## R12 (rules.md lines 622-623): a lambda's `changes` is "checked against its body both ways (E2105)", so
## naming a variable it never changes is refused as an unnamed change is. Message in the shape of AN-76
## part 6's "`tick` says `changes LOG` and never changes it" (no lambda message is given in the rules).

fn main():
    score = 0
    show = fn() changes score: println(score)    #$ error[E2105]: this function says `changes score` and never changes it; remove `score` from its first line: `show = fn(): println(score)`
    show()
