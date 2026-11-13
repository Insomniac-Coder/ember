# Clause BRW-1.1: "At every point of the program a place is reached through any number of read-only
# views, or through exactly one name that may change it, as `[BRW-17]` to `[BRW-20]` and the two rules
# below say; every one of them is checked when compiling, and nothing is checked while the program
# runs."
#$ test: run-pass
#$ rules: BRW-1, BRW-2, BRW-12, SPN-1
#$ stdout: 5 5 5 5
#$ stdout: 6
# Four read-only views of one enemy are used together; then one `ref mut` name changes it, alone.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=5)]
    a = enemies[0]
    b = ref enemies[0]
    s = enemies.as_span()
    println(a.hp, b.hp, s[0].hp, enemies[0].hp)
    boss = ref mut enemies[0]
    boss.hp += 1
    println(enemies[0].hp)
