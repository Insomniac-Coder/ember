# A3 attack: a view of b.units passed into a write through a; a and b may be one squad (rule 1).
struct Squad:
    units: Array[int]

fn merge(mut a: Squad, b: Squad):
    a.units.extend(b.units[0..2])

fn main():
    squads = [Squad(units=[1, 2, 3])]
    merge(squads[0], squads[0])
    println(squads[0].units.len())
