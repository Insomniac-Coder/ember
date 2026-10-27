#$ test: compile-fail
#$ rules: R2, R8
#$ help: use `n` before line 16, or keep your own copy: write `n = copy of p.name` in place of line 15
## R2, R8 (b): a name for a part views into its owner; handing the owner over while the name is still
## used would leave it pointing at a value that moved (E3060; AN-35's refused program and message).

struct Player:
    name: String
    inventory: Array[String]

fn keep(owns p: Player):
    pass

fn log_and_keep(owns p: Player):
    n = p.name
    keep(p)
    println(n)                 #$ error[E3060]: `n` views `p.name` (line 15), and `keep(p)` hands `p` over on line 16 while `n` is used on line 17

fn main():
    log_and_keep(Player(name=String("ann"), inventory=[]))
