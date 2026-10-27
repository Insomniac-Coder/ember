#$ test: compile-fail
#$ rules: R8, R6
#$ help: es[0].heal()
## R8 (d), verdict 13: a method that changes its object, called on a temporary that ends with the line,
## is refused when the change is the call's only effect. AN-10, point 6.

@derive(Copy)
struct Enemy:
    hp: int

    fn heal() changes hp:
        this.hp += 10

fn enemy_at(xs: Array[Enemy], i: int) -> Enemy:
    return xs[i]

fn main():
    es = [Enemy(hp=1)]
    enemy_at(es, 0).heal()    #$ error[E2140]: `heal` changes the enemy that `enemy_at(es, 0)` returns, a copy that ends on this line, so the change would be lost; to heal the enemy in `es` write `es[0].heal()`, or let `enemy_at` give back the enemy itself: `fn enemy_at(ref mut xs: Array[Enemy], i: int) -> ref mut Enemy:` with `return ref mut xs[i]`, called `enemy_at(ref mut es, 0).heal()`
