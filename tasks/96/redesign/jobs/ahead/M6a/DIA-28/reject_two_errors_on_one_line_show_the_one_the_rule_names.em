# Clause DIA-28.1: "Where both fixes compile, the rule that names the case decides: a position used
# after a removal is `E3060`, not `E3022`; a view used after a call whose callee ends what it points
# into, where `E3021` and `E3031` both fit, is `E3031`."
#$ test: compile-fail
#$ rules: DIA-28, BRW-16, BRW-21
# Each line claims the one code the clause names, so the other, if shown too, is unclaimed and
# fails the file. The `E7018`/`E3020` and `E3020`/`E7007` cases need locks and scoped tasks (M9):
# not here. No text in the spec for these programs: code only.

struct Enemy:
    hp: int

static mut LOG: Array[String] = []

fn note(s: StrView):
    LOG.clear()
    println(s)

fn main():
    enemies = [Enemy(hp=3), Enemy(hp=4)]
    boss = ref mut enemies[0]
    enemies.remove(0)
    boss.hp = 1    #$ error[E3060]
    LOG.push(owns String("a"))
    note(LOG[0])    #$ error[E3031]
