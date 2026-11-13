# Clause BRW-14.2: "and so is any other change through one that is plain (`[BRW-15]`)."
#$ test: compile-fail
#$ rules: BRW-14, BRW-15
# The spec gives no message text for these programs (`[BRW-15]`'s examples are other programs):
# codes only.

struct Enemy:
    hp: int

fn hurt(e: Enemy):
    e.hp -= 1    #$ error[E3023]

fn main():
    enemies = [Enemy(hp=3)]
    for e in enemies:
        e.hp = 0    #$ error[E3023]
    o: Option[Enemy] = Some(Enemy(hp=1))
    if o exists as q:
        q.hp += 1    #$ error[E3023]
    hurt(enemies[0])
