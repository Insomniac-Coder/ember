# Clause DRP-8.1: "a list cleared, a value replaced, a local ending with its function"; "and a
# function that may end an `Enemy` changes `LOG`"
#$ test: compile-fail
#$ rules: DRP-8, BRW-18, EFF-1
# A replaced value, a local that ends with its function, and a function given the list: each may end
# an `Enemy` inside the walk of `LOG`. The function-value and `owns dyn I` cases need capture words
# (M8) and `owns dyn` (M11): not here. No text in the spec for these: code only.

static mut LOG: Array[String] = []

class Enemy:
    hp: int = 1

    fn drop():
        LOG.push(owns String("enemy gone"))

fn spawn_and_forget():
    e = Enemy()
    println(e.hp)

fn wipe(ref mut es: Array[Enemy]):
    es.clear()

fn main():
    boss = Enemy()
    enemies = [Enemy()]
    LOG.push(owns String("start"))
    for line in LOG:
        println(line)
        boss = Enemy()    #$ error[E3020]
        spawn_and_forget()    #$ error[E3020]
        wipe(ref mut enemies)    #$ error[E3020]
    println(boss.hp)
