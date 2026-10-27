#$ test: compile-fail
#$ rules: R5
#$ help: write `fn tick():`
## R5, verdict 86: `changes` is not a word of Ember: the compiler works out what a function changes from
## its body. A first line that still writes it is refused with the line to write (E0100, a clause after a
## function's brackets that Ember does not have; `V5/rules.md` section 3's message in this program's
## names). The nearest program that is allowed is `accept_a_function_changes_a_module_variable_with_no_word`.

static mut FRAME: int = 0

fn tick() changes FRAME:       #$ error[E0100]: `changes FRAME` is not Ember: the compiler works out what `tick` changes from its body
    FRAME += 1

fn main():
    tick()
    println(FRAME)
