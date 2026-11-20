# Clause BRW-11.4: "Parts whose positions are not known when compiling (`ref mut log[i]` beside
# `ref mut log[j]`, or beside `log`) stay refused (`E3022`)."
#$ test: compile-fail
#$ rules: BRW-11, MONO-10, BRW-5
# The spec gives no text: code only.

fn add_both(ref mut a: int, ref mut b: int):
    a += 1
    b += 1

fn add_to_all(ref mut a: int, ref mut all: Array[int]):
    a += 1
    all.push(owns 0)

fn main():
    log = [1, 2]
    i = 0
    j = 0
    add_both(ref mut log[i], ref mut log[j])    #$ error[E3022]
    add_to_all(ref mut log[i], ref mut log)    #$ error[E3022]
