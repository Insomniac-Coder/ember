#$ test: compile-fail
#$ rules: RC-1
# `[RC-1]` — a name a pattern gives, which holds no value of its own that its line could make an owner
# of, cannot be shared yet.

class Counter:
    n: int = 0

fn main():
    pair = (Counter(), 1)
    (c, k) = pair
    d = shares mut c    #$ error[E0900]: `shares mut c` of a name no line gives a value of its own is not built by this compiler yet
    println(d.n, k)
