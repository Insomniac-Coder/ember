#$ test: compile-fail
#$ rules: EXP-5, BRW-19
#$ help: keep the value in a name first
# `[EXP-5]` — a method that may change its object, called on a temporary that ends with the line, is
# `E2140` when that change is the call's only effect, since the change would be lost
# (`enemy_at(es, 0).heal()`).

struct Enemy:
    hp: int

    fn heal():
        this.hp += 1

fn enemy_at(es: Array[Enemy], i: int) -> Enemy:
    return copy of es[i]

fn main():
    es = [Enemy(hp=1)]
    enemy_at(es, 0).heal()    #$ error[E2140]: `heal` changes `enemy_at(es, 0)`, a temporary that ends with the line, so the change would be lost
