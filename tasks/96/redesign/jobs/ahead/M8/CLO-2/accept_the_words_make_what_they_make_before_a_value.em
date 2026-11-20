# Clause CLO-2.2: "Each word means what it means before a value (`[OWN-13]`): `copy of` a copy of its
# own; `owns` handed over, for every type, numbers included (...), and the lambda uses it under the
# same name; `shares`, `shares mut` and `thread_shared` one more owner"
#$ test: run-pass
#$ rules: CLO-2, CLO-5, OWN-13
#$ stdout: 6
#$ stdout: 5 5
#$ stdout: 3
# `owns count` hands a number over; `shares p` is one more owner, which sees the other owner's value;
# `copy of xs` is a list of the function's own.

class Player:
    hp: int = 5

fn main():
    count = 5
    f = fn() owns count => count + 1
    println(f())
    p = shares Player()
    g = fn() shares p => p.hp
    println(g(), p.hp)
    xs = [1, 2]
    h = fn() copy of xs => xs.len() + 1
    xs.push(owns 3)
    println(h())
