# A3 attack: a view into one squad's list, passed with the list of squads to a function that grows
# every squad's list. The first extend may move squads[0].units; `picked` then points at freed memory.
struct Squad:
    units: Array[int]

fn reinforce(mut squads: Array[Squad], picked: Span[int]):
    for s in squads.iter_mut():
        s.units.extend(picked)

fn main():
    squads = [Squad(units=[1, 2, 3]), Squad(units=[4])]
    reinforce(squads, squads[0].units[0..2])
    println(squads[1].units.len())
