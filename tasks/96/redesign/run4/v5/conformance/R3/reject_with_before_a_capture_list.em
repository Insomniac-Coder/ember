#$ test: compile-fail
#$ rules: GRM-2
#$ help: write `show = fn() copy of score: println(score)`
## design rules: R3, R12
## R3, R12: what a function keeps follows its brackets with no `with` (verdict 83: `fn() copy of
## score`); `with` is only the block (`with g = m.lock():`). A `with` before the captures is a clause
## Ember does not have (E0100, `V5/rules.md` section 3's message). The nearest program that is allowed
## is `accept_a_captured_copy_is_the_function_values_own`.

fn main():
    score = 1
    show = fn() with copy of score: println(score)     #$ error[E0100]: a function's captures follow its brackets with no `with`
    show()
