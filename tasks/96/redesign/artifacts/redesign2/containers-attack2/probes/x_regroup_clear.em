# A3 attack: a view of b's units kept across a.units.clear(), when a and b are one squad.
struct Unit:
    name: String
    hp: int

struct Squad:
    units: Array[Unit]

fn regroup(mut a: Squad, b: Squad) -> int:
    first_two = b.units[0..2]
    a.units.clear()
    return first_two[0].name.len()

fn main():
    s = Squad(units=[Unit(name="ann", hp=1), Unit(name="bob", hp=2)])
    println(regroup(s, s))
