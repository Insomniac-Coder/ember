# Clause BRW-17.2: "unless the compiler knows they differ (types of which neither can hold a value of
# the other, for classes neither deriving from the other; a fresh object; inside `if a is not b:`)."
#$ test: run-pass
#$ rules: BRW-17, BCK-3, BCK-7, RC-1
#$ stdout: ann
#$ stdout: ann 0
#$ stdout: bob
# One function per proof: the change cannot reach the text `w` views, so `w` lives on.

class Player:
    name: String

class Monster:
    name: String

fn other_type(a: shares mut Player, ref mut m: shares mut Monster):
    w = a.name.trim()
    m.name = String("x")
    println(w)

fn fresh_object(a: shares mut Player):
    w = a.name.trim()
    c = Player(name=String("c"))
    c.name = String("x")
    println(w, c.name.len() - 1)

fn proved_apart(a: shares mut Player, ref mut b: shares mut Player):
    w = a.name.trim()
    if a is not b:
        b.name = String("x")
    println(w)

fn main():
    p = shares mut Player(name=String("ann"))
    m = shares mut Monster(name=String("m"))
    other_type(p, ref mut m)
    fresh_object(p)
    q = shares mut Player(name=String("bob"))
    proved_apart(q, ref mut p)
