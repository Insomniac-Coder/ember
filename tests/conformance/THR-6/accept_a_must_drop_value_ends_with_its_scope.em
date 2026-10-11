#$ test: run-pass
#$ rules: THR-6
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ 2 4 true
#$ released 5
#$ released 3
#$ released 2
#$ released 1

# `[THR-6]` — a `@must_drop` value lives in locals, is lent and passed on, sits
# in a tuple or an `Option` (which become `@must_drop` too), and its drop runs
# when its scope ends. A function may return one a caller made.

@must_drop
struct Guard:
    id: int

    fn drop(mut self):
        println(f"released {self.id}")

fn look(g: Guard) -> int:
    return g.id

fn consume(owned g: Guard) -> Guard:
    return g

fn main():
    g = Guard(1)
    println(look(g))
    h = consume(Guard(2))
    pair = (Guard(3), 4)
    o: Option[Guard] = Some(Guard(5))
    println(h.id, pair.1, o.is_some())
