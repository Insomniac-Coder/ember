#$ test: compile-fail
#$ rules: R12
#$ help: write `return fn() owns n:` in place of this line
## Verdicts 80 and 84, R12 (rules.md lines 761-765): "A function that outlives the function that made it
## takes what it keeps with `owns`. `return fn(): n += 1` in `make_counter`, whose local `n` ends when it
## returns, is refused (E3026)". `n` is an `int`: `owns` hands over every type the same way, numbers
## included (verdict 84), so the message gives verdict 80's own line. rules.md section 3's line 1095
## second message, in this program's lines (its lambda has a block). The nearest allowed program is
## accept_a_returned_function_owns_the_variable_it_keeps.

fn make_counter() -> fn():
    n = 0
    return fn():    #$ error[E3026]: the returned function changes `n`, which ends when `make_counter` returns; let the function own it: write `return fn() owns n:` in place of this line
        n += 1
        println(n)

fn main():
    next = make_counter()
    next()
    next()
