#$ test: compile-fail
#$ rules: GRM-43, OWN-8
## `[OWN-8]` — `copy of` before what it cannot copy as itself is `E2116`: a type with no `Clone`
## (a struct with a `drop` of its own derives none unless it asks), and a view, whose text is
## another's. `copy of` makes a new object of a class; this compiler's `Clone` of a class object is
## 0.9.10's, a copy of the handle naming the same object, so `copy of` a value holding one is `E0900`
## (`[CLI-19]`), not a copy that is none.

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
    q = copy of p    #$ error[E0900]: `copy of` a `Player`, which holds a class object, is not built by this compiler yet
    line = String("a b")
    w: StrView = line.as_str()
    v = copy of w    #$ error[E2116]: `w` is a view of what another value holds, so `copy of` cannot copy it
    c = Counter(n=1)
    d = copy of c    #$ error[E0900]: `copy of` a `Counter`, which holds a class object, is not built by this compiler yet
    t = Team(lead=Counter(n=2))
    u = copy of t    #$ error[E0900]: `copy of` a `Team`, which holds a class object, is not built by this compiler yet
    println(g.fd, q.hp, v, d.n, u.lead.n)
