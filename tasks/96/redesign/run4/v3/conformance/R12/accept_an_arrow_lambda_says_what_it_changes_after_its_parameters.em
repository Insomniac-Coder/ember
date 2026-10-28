#$ test: run-pass
#$ rules: R12, R5
#$ stdout: 2 1
## Verdict 45, R12 (rules.md lines 520-521, 543-549): `fn(e) changes removed => ...` names `update`'s
## real `removed`; `retain` (`[C, F: fn(T) changes C -> bool]`) counts as changing it, and `update`, which
## wrote the lambda, names nothing for it.

struct Enemy:
    hp: int

fn keep(e: Enemy, ref mut removed: int) -> bool:
    if e.hp > 0:
        return true
    removed += 1
    return false

fn update(ref mut enemies: Array[Enemy]) -> int:
    removed = 0
    enemies.retain(fn(e) changes removed => keep(e, ref mut removed))
    return removed

fn main():
    es = [Enemy(hp=1), Enemy(hp=0), Enemy(hp=0)]
    n = update(ref mut es)
    println(n, es.len())
