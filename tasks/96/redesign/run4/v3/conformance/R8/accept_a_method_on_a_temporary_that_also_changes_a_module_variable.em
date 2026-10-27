#$ test: run-pass
#$ rules: R8, R5
#$ stdout: 1 1
## R8 (d): E2140 is only for a call whose only effect is the change of the temporary; a method whose
## first line also changes `LOG` has an effect that is not lost.

static mut LOG: Array[String] = []

@derive(Copy)
struct Enemy:
    hp: int

    fn heal_and_log() changes hp, LOG:
        this.hp += 10
        LOG.push(String("healed"))

fn enemy_at(xs: Array[Enemy], i: int) -> Enemy:
    return xs[i]

fn main() changes LOG:
    es = [Enemy(hp=1)]
    enemy_at(es, 0).heal_and_log()
    println(LOG.len(), es[0].hp)
