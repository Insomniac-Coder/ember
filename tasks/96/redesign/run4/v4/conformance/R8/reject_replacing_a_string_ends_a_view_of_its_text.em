#$ test: compile-fail
#$ rules: R8
#$ help: n = String(boss.name.trim())
## R8 (b): a view into a value's own contents (a `String`'s text) ends at any change that may reshape
## what it points into; replacing the `String` frees its old text. AN-01, point 6.

struct Enemy:
    name: String
    hp: int

fn main():
    enemies = [Enemy(name=String(" orc "), hp=10)]
    boss = ref mut enemies[0]
    n = boss.name.trim()
    boss.name = String("x")
    println(n.len())    #$ error[E3060]: `n` is a view of `enemies[0].name`, whose old text line 15 frees, and `n` is used on line 16; use `n` before line 15, or keep your own copy: write `n = String(boss.name.trim())` in place of line 14
