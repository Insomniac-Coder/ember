#$ test: run-pass
#$ rules: R2, R4
#$ stdout: ann
#$ 0
## R2: a name bound to a part (`n = p.name`) is a read-only view of it, never a hand-over and never a
## silent copy, so `p` stays whole and may be handed over after `n`'s last use (AN-35's NEW
## `log_and_keep`).

struct Player:
    name: String
    inventory: Array[String]

fn keep(owns p: Player) -> int:
    return p.inventory.len()

fn log_and_keep(owns p: Player) -> int:
    n = p.name
    println(n)
    return keep(p)

fn main():
    println(log_and_keep(Player(name=String("ann"), inventory=[])))
