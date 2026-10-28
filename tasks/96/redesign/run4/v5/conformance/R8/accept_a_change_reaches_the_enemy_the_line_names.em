#$ test: run-pass
#$ rules: BRW-16, LT-1
#$ stdout: 21 31
## design rules: R8, R6
## R8 (d), verdict 13: both ways out of E2140 heal the enemy in `es`; a copy of your own is healed on its
## own. AN-10's second NEW program.

@derive(Copy)
struct Enemy:
    hp: int

    fn heal():
        this.hp += 10

fn enemy_at(ref mut xs: Array[Enemy], i: int) -> ref mut Enemy:
    return ref mut xs[i]

fn main():
    es = [Enemy(hp=1)]
    es[0].heal()
    enemy_at(ref mut es, 0).heal()
    e = copy of es[0]
    e.heal()
    println(es[0].hp, e.hp)
