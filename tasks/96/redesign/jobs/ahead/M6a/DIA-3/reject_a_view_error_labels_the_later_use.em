# Clause DIA-3.1: "Ownership and view errors (§XVII.6.1) label the later use that makes the refusal
# ("used here") and give the fix of their shape."
#$ test: compile-fail
#$ rules: DIA-3, DIA-7
# Three shapes whose refusal rests on a later use (B7, O1, B3): each claims the label's words.

struct Enemy:
    hp: int

class Player:
    hp: int = 0

fn main():
    names = [String("a")]
    v = names[0]
    names.clear()
    println(v)    #$ error[E3060]: used here
    a = Player()
    b = a
    println(a.hp)    #$ error[E3040]: used here
    println(b.hp)
    enemies = [Enemy(hp=3)]
    boss = enemies[0]
    enemies[0].hp = 1    #$ error[E3021]: used here
    println(boss.hp)
