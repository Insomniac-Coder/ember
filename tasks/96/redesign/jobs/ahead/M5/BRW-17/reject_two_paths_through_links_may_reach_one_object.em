# Clause BRW-17.2: "Two paths may reach the same object when they lead to values of one type and at
# least one goes through a link or a kept view, unless the compiler knows they differ (...)."
#$ test: compile-fail
#$ rules: BRW-17, BCK-3, BCK-7, RC-1, FN-1
# `a` and `b` are links to `Player`s, which may be one player: replacing `b`'s name may end the text
# `w` views. In `main` they are one; in `rename` the compiler cannot know. Codes only.

class Player:
    name: String

fn rename(a: shares mut Player, ref mut b: shares mut Player):
    w = a.name.trim()
    b.name = String("x")
    println(w)    #$ error[E3060]

fn main():
    p = shares mut Player(name=String("ann"))
    q = shares mut p
    w = p.name.trim()
    q.name = String("x")
    println(w)    #$ error[E3060]
    r = shares mut Player(name=String("bob"))
    rename(p, ref mut r)
