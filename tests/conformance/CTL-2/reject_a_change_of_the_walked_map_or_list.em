#$ test: compile-fail
#$ rules: CTL-2, BRW-18, DIA-25
# `[CTL-2]` — while a loop walks a collection, nothing changes it but the loop's own handle: `counts[k] *= 2`
# inside `for k, v in counts:` and `xs[0] = 5` inside `for x in xs:` are `E3020`, whose message gives the
# loop's own handle and the line through it; anything else goes after the loop.

struct Enemy:
    hp: int

fn main():
    counts = Map[String, int]()
    counts.insert(owns String("a"), owns 1)
    for k, _v in counts:
        counts[k] *= 2    #$ error[E3020]: `counts[k] *= 2` (line 14) changes `counts`, whose keys this loop walks (line 13); change each value through the loop's own handle: write `for k, ref mut v in counts:` in place of line 13 and `v *= 2` in place of line 14
    for name in counts:
        counts[name] = 0    #$ error[E3020]: `counts[name] = 0` (line 16) changes `counts`, whose keys this loop walks (line 15); change each value through the loop's own handle: write `for name, ref mut v in counts:` in place of line 15 and `v = 0` in place of line 16
    xs = [1, 2]
    for _x in xs:
        xs[0] = 5    #$ error[E3020]: `xs[0] = 5` (line 19) writes an element of `xs`, which this loop walks (line 18); to change the element the loop is on, write `for ref mut x in xs:` in place of line 18 and `x = 5` in place of line 19
    enemies = [Enemy(hp=1)]
    for e in enemies:
        enemies[0].hp = e.hp + 1    #$ error[E3020]: `enemies[0].hp = e.hp + 1` (line 22) writes an element of `enemies`, which this loop walks (line 21); to change the element the loop is on, write `for ref mut e in enemies:` in place of line 21 and `e.hp = e.hp + 1` in place of line 22
        _gone = enemies.pop()    #$ error[E3020]: `_gone = enemies.pop()` (line 23) changes `enemies`, which this loop walks (line 21); change it after the loop: move line 23, `_gone = enemies.pop()`, below the loop
