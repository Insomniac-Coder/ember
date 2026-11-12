#$ test: compile-fail
#$ rules: OWN-11, EXP-6, DIA-26
# `[OWN-11]` — the messages of a part handed over on its own say where it went and the line to write:
# `p.name` used again is `E3040` ("was handed over to `names` on line 22; give it a new value first, or
# keep a copy: `names.push(owns copy of p.name)`"); a field of a class object cannot be handed out and leave
# a hole, `E3013` ("... `names.push(owns mem.take(ref mut q.name))`"); and a hand-over out of a name that
# does not own the value, `E3013` ("`v.push(owns template)` would hand over the caller's player, but
# `template` only reads it; to put a player of your own in the list write `v.push(owns copy of template)`").

struct Player:
    name: String
    hp: int

class Mob:
    name: String
    hp: int

fn first(template: Player) -> Array[Player]:
    v: Array[Player] = []
    v.push(owns template)    #$ error[E3013]: `v.push(owns template)` would hand over the caller's player, but `template` only reads it; to put a player of your own in the list write `v.push(owns copy of template)`
    return v

fn main():
    p = Player(name = "a", hp = 1)
    names: Array[String] = []
    names.push(owns p.name)
    println(p.name)    #$ error[E3040]: `p.name` was handed over to `names` on line 26; give it a new value first, or keep a copy: `names.push(owns copy of p.name)`
    q = Mob(name = "b", hp = 2)
    names.push(owns q.name)    #$ error[E3013]: `q` is a `Mob` object, which other names may reach, so `q.name` cannot be taken out and leave a hole; take it and leave a value in its place: `names.push(owns mem.take(ref mut q.name))`
    println(first(p).len(), names.len(), q.hp)
