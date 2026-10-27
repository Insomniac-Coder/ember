#$ test: compile-fail
#$ rules: R3, R12, R1
#$ help: to keep your own, write `fn() copy of score` in place of `fn() owns score` on line 11
## R3, R12, R1: `owns` hands over every type the same way, numbers included (verdict 84): after
## `fn() owns score` the function that made the lambda no longer uses `score`, and a later use is
## refused (E3040, `V5/rules.md` section 3's message in this program's names). `copy of` copies, and the
## giver keeps its own (`accept_a_captured_copy_is_the_function_values_own`).

fn main():
    score = 1
    show = fn() owns score: println(score)
    show()
    println(score)             #$ error[E3040]: `score` was handed over to the function on line 11 (`fn() owns score`) and is used on line 13
