#$ test: compile-fail
#$ rules: GRM-43, OWN-8
## `[OWN-8]` — `copy of` before what it cannot copy as itself is `E2116`: a type with no `Clone` (a
## struct with a `drop` of its own derives none unless it asks, and a class has none unless it says
## so). A read-only view is `Copy`, so its copy is one more view; a class's `Clone` makes a new object.

struct File:
    fd: int = 0

    fn drop():
        println("closed")

class Player:
    hp: int = 0

@derive(Clone)
class Counter:
    n: int = 0

@derive(Clone)
struct Team:
    lead: Counter

fn main():
    f = File(fd=3)
    g = copy of f    #$ error[E2116]: `File` has no `Clone`, so `copy of` cannot make one
    p = Player(hp=3)
    q = copy of p    #$ error[E2116]: `Player` has no `Clone`, so `copy of` cannot make one
    line = String("a b")
    w: StrView = line.as_str()
    v = copy of w
    c = Counter(n=1)
    d = copy of c
    t = Team(lead=Counter(n=2))
    u = copy of t
    println(g.fd, q.hp, v, d.n, u.lead.n)
