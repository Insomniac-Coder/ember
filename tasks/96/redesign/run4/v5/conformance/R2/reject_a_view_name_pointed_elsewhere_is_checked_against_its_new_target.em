#$ test: compile-fail
#$ rules: BRW-17
#$ help: use `n` before line 16, or name it again after it: `n = q.name`
## design rules: R2, R8
## R2, R8 (g): after `n = q.name` the name views `q.name`, and the promise of a read-only name now
## covers `q.name`. Replacing a `String` frees its text (a reshape, `V4/rules.md` section 0), so it
## ends `n` instead of being refused at the change: `n`'s later use is refused (E3060).

struct Player:
    name: String

fn main():
    p = Player(name=String("ann"))
    q = Player(name=String("bo"))
    n = p.name
    n = q.name
    q.name = String("cy")
    println(n)                 #$ error[E3060]: `q.name = String("cy")` (line 16) replaces the text `n` views (`n = q.name`, line 15), and `n` is used on line 17
