#$ test: compile-fail
#$ rules: HEAP-3, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14
#$ profiles: debug, development, release

# `[RC-2e]` — a loop's name over a list of `shares` owners is a view, and a normal closure keeps it as
# one (`ref shares mut Token`): a change through it is refused, as through any view that only reads.
struct Token:
    value: i32

fn main():
    tokens: Array[shares mut Token] = Array[shares mut Token]()
    tokens.push(owns shares mut Token(7))
    for token in tokens:
        fn task_fn():
            token.value = 8 #$ error[E3021]: cannot write through a shared reference
        task = task_fn
        task()
