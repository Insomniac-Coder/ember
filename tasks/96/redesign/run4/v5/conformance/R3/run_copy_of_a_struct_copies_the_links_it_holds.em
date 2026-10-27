#$ test: run-pass
#$ rules: OWN-8, RC-1, WK-11
#$ stdout: 5
#$ 5
#$ player ended
#$ done
## design rules: R3, R9, R11
## R3, R9, R11 and `V5/rules.md` section 2 ("a count +1 / -1"; "one add and one subtract of the
## target's weak count"): `copy of` a struct that holds links copies the links (verdict 79): `t2.captain`
## is one more owner of the same player and `t2.scout` one more link to it, so a change through `t2` is
## seen through `t`, and the player ends once, with its last owner (`t2`), not before (completeness gap
## G17). Verified by its output and a clean AddressSanitizer and leak run (one block, freed once).

import std.mem

class Player:
    hp: int = 10

    fn drop():
        println("player ended")

@derive(Clone)
struct Team:
    captain: shares mut Player
    scout: weakref Player = None

fn main():
    p = shares mut Player()
    t = Team(captain=shares mut p)
    t.scout = weakref p
    t2 = copy of t
    t2.captain.hp = 5
    println(t.captain.hp)
    mem.drop(p)
    mem.drop(t)
    if t2.scout exists as s:
        println(s.hp)
    mem.drop(t2)
    println("done")
