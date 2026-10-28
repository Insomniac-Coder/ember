#$ test: compile-fail
#$ rules: R2, R8
#$ help: change it after `n`'s last use, or write `n = ref mut q.name` and change it through `n`: `n = String("cy")`
## R2, R8 (g): after `n = q.name` the name views `q.name`, and the promise of a read-only name now
## covers `q.name`: replacing it while `n` is still used is refused (E3021).

struct Player:
    name: String

fn main():
    p = Player(name=String("ann"))
    q = Player(name=String("bo"))
    n = p.name
    n = q.name
    q.name = String("cy")      #$ error[E3021]: `q.name = String("cy")` (line 15) changes the text `n` reads (`n = q.name`, line 14), and `n` is used on line 16
    println(n)
