# Clause BRW-17.5: "Adding to a list moves only what it stores inline: (...); one to a value stored
# inline in the list (a struct element) is."
#$ test: compile-fail
#$ rules: BRW-17, LT-46, BCK-3
# The same `Watcher` shape as accept_adding_keeps_views_of_class_objects, over struct elements. Codes
# only.

struct Enemy:
    hp: int

struct Grip:
    target: sharedref mut Enemy

fn main():
    enemies = [Enemy(hp=1)]
    g = Grip(target=sharedref mut enemies[0])
    enemies.push(owns Enemy(hp=2))
    g.target.hp -= 1    #$ error[E3060]
