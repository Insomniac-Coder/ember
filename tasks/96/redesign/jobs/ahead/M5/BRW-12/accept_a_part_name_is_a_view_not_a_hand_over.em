# Clause BRW-12.2: "A name bound to a part (`n = p.name`, `boss = enemies[0]`), or to a call whose
# result is a view (`ref T`, `StrView`, `Span[T]`, a `ref` or `ref mut` place, `[LT-1]`), such as
# `word = line.to_lower().trim()` or `best = a.as_span()`, is a read-only view of it: never a
# hand-over, and never a silent copy"
#$ test: run-pass
#$ rules: BRW-12, BRW-19, LT-1
#$ stdout: orc 10
#$ stdout: orc
#$ stdout: elf 2
#$ stdout: hello there
#$ stdout: 3 3
# Nothing is handed over: `enemies` stays whole and usable beside the views.

struct Enemy:
    name: String
    hp: int

fn main():
    enemies = [Enemy(name=String("orc"), hp=10), Enemy(name=String("elf"), hp=7)]
    boss = enemies[0]
    println(boss.name, boss.hp)
    println(enemies[0].name)
    n = enemies[1].name
    println(n, enemies.len())
    line = String("  Hello There ")
    word = line.to_lower().trim()
    println(word)
    a = [3, 1, 2]
    best = a.as_span()
    println(best.len(), a[0])
