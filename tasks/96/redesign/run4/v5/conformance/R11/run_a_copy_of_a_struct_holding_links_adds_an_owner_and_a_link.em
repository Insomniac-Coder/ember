#$ test: run-pass
#$ rules: WK-11, OWN-8, RC-1
#$ stdout: 10
#$ stdout: done
#$ stdout: player ends
## design rules: R11, R3, R9
## R3, R11 (verdict 79): "`copy of` a struct that holds links copies the links": `copy of t` is a new
## `Team` whose `captain` is one more owner of the same player (+1) and whose `scout` is one more link
## to it (one more weak count). When `keep_copy` returns, `p` and `t` end, and the copy's `captain`
## keeps the player alive, so its `scout` still opens; the player ends with the copy, at the end of
## `main`. Checked under AddressSanitizer: no report (a copy that copied the pointers without the
## counts would free the player twice).

class Player:
    hp: int = 10

    fn drop():
        println("player ends")

@derive(Clone)
struct Team:
    captain: shares Player
    scout: weakref Player

fn keep_copy() -> Team:
    p = Player()
    t = Team(captain=shares p, scout=weakref p)
    return copy of t

fn main():
    t2 = keep_copy()
    if t2.scout exists as s:
        println(s.hp)
    println("done")
