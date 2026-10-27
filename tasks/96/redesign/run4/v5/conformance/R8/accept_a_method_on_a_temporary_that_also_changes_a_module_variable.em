#$ test: run-pass
#$ rules: BRW-16, FN-4
#$ stdout: 1 1
## design rules: R8, R5
## R8 (d): E2140 is only for a call whose only effect is the change of the temporary; a method whose
## body also changes `LOG` has an effect that is not lost (worked out, verdict 86).

static mut LOG: Array[String] = []

@derive(Copy)
struct Enemy:
    hp: int

    fn heal_and_log():
        this.hp += 10
        LOG.push(String("healed"))

fn enemy_at(xs: Array[Enemy], i: int) -> Enemy:
    return xs[i]

fn main():
    es = [Enemy(hp=1)]
    enemy_at(es, 0).heal_and_log()
    println(LOG.len(), es[0].hp)
