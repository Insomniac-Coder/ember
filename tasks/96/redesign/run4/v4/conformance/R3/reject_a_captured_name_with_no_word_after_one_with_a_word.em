#$ test: compile-fail
#$ rules: R3
#$ help: say what the function does with `score`: `with copy of name, copy of score`, or leave `score` out to use the real `score`
## R3: in a list of names declared without types each word belongs to the one name after it, and a
## name with no word may not follow a name with a word (E0116; `V4/rules.md` section 3's message).

fn main():
    name = String("ann")
    score = 3
    show = fn() with copy of name, score: println(name, score)     #$ error[E0116]: `copy of` here belongs to `name` only
    show()
