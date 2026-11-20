#$ test: compile-fail
#$ rules: LT-3, STA-1, BRW-16, BRW-17, BRW-1, CTL-2
# `[LT-3]` — a `static mut` is a place with one owner, the program, that a function naming it reaches
# as it reaches a local of its own: each use of `ALL` is that one place, so a position of `ALL[0]`
# meets a later `ALL.clear()` (`[BRW-16]`), a walk of `LOG` meets a push to it (`[CTL-2]`), and a
# read-only view of `ALL[0]` meets a write of it (`[BRW-1]`), with the messages a local gets.

struct Enemy:
    hp: int

static mut ALL: Array[Enemy] = [Enemy(hp=0)]
static mut LOG: Array[int] = [1, 2]

fn ended():
    a = ref mut ALL[0]
    ALL.clear()
    a.hp = 5    #$ error[E3060]: `ALL.clear()` (line 16) may end the enemy `a` names (line 15), and `a` is used on line 17; use `a` before line 16, or name the enemy again after it: `a = ref mut ALL[0]`

fn walked():
    for x in LOG:
        LOG.push(owns x)    #$ error[E3020]: `LOG.push(owns x)` (line 21) adds to `LOG`, which this loop walks (line 20)

fn viewed():
    v = ALL[0]
    ALL[0].hp = 3    #$ error[E3021]: `ALL[0].hp = 3` (line 25) changes the enemy `v` reads (`v = ALL[0]`, line 24), and `v` is used on line 26
    println(v.hp)

fn main():
    ended()
    walked()
    viewed()
