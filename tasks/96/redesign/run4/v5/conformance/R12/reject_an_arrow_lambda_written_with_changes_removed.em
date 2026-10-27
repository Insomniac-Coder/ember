#$ test: compile-fail
#$ rules: R12, R5
#$ help: enemies.retain(fn(e) => keep(e, ref mut removed))
## Verdict 86 (verdict 45 keeps its meaning without the clause), R12 (rules.md lines 670-672): a lambda
## written with `=>` writes nothing about what it changes either: "`enemies.retain(fn(e) => keep(e, ref mut
## removed))` adds to `update`'s `removed`". A clause between its parameters and `=>` is E0100 (rules.md
## section 3, line 1051). The nearest allowed program is
## accept_an_arrow_lambda_changes_a_variable_it_uses.

struct Enemy:
    hp: int

fn keep(e: Enemy, ref mut removed: int) -> bool:
    if e.hp > 0:
        return true
    removed += 1
    return false

fn update(ref mut enemies: Array[Enemy]) -> int:
    removed = 0
    enemies.retain(fn(e) changes removed => keep(e, ref mut removed))    #$ error[E0100]: `changes removed` is not Ember: the compiler works out what this function changes from its body; write `enemies.retain(fn(e) => keep(e, ref mut removed))`
    return removed

fn main():
    es = [Enemy(hp=1), Enemy(hp=0), Enemy(hp=0)]
    n = update(ref mut es)
    println(n, es.len())
