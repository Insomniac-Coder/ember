#$ test: compile-fail
#$ rules: OWN-11
#$ help: to put a player of your own in the list write `v.push(copy of template)`
## design rules: R1, R2, R4
## R1, R2, R4: a plain parameter is a read-only view of the caller's value, so storing it in a list
## would hand over what it does not own (E3013; AN-35's `spawn_all`, with the message of `V4/rules.md`
## section 3).

@derive(Clone)
class Player:
    name: String

fn spawn_all(template: Player, n: int) -> Array[Player]:
    v: Array[Player] = []
    for _ in 0..n:
        v.push(template)       #$ error[E3013]: `v.push(template)` would hand over the caller's player, but `template` only reads it
    return v

fn main():
    p = Player(name=String("ann"))
    println(spawn_all(p, 3).len())
