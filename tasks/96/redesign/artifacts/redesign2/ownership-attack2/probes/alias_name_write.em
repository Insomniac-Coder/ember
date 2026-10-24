# Attack on names-follow-values rule 2: a name for a field, the field replaced through a second
# handle that may be the same object. Python (py_attack_output.txt): alias_write_same ann.
class Player:
    name: String = "ann"

fn rename(p: Player, q: Player) -> String:
    old = p.name
    q.name = "bob"
    return old

fn main():
    x = Player()
    println(rename(x, x))
