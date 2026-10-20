#$ test: run-pass
#$ rules: GRM-19, GRM-6, CTL-0
#$ profiles: debug, development, release
#$ stdout: 0.25
#$ stdout: 4 -1
#$ stdout: 3
## `condition := pattern "=" expression` (D-763) — `if Some(p) = h:` runs its block when the value
## matches, with the pattern's names bound there; `elif` and `else` follow as for any `if`.
## `while Some(x) = it.next():` runs until the value does not match, and its `else` then runs; a
## `break` leaves without it.

class Player:
    health: f32 = 1.0

fn first_even(xs: Array[int]) -> int:
    it = xs.iter()
    while Some(x) = it.next():
        if x % 2 == 0:
            return x
    else:
        return -1
    return -2

fn count_until_three(xs: Array[int]) -> int:
    seen = 0
    it = xs.iter()
    while Some(x) = it.next():
        if x == 3:
            break
        seen += 1
    else:
        seen = 100
    return seen

fn main():
    h: Option[Player] = Player()
    if Some(p) = h:
        p.health -= 0.5
    if Some(p) = h: p.health -= 0.25
    n: Option[int] = None
    if Some(v) = n:
        println(v)
    elif Some(p) = h:
        println(p.health)
    else:
        println("none")
    println(first_even([1, 3, 4, 5]), first_even([1, 3]))
    println(count_until_three([1, 2, 5, 3, 9]))
