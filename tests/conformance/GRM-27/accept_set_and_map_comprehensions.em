#$ test: run-pass
#$ rules: GRM-27, TYP-38
#$ stdout: {3: 9, 1: 1, 2: 4} {1, 0}
#$ {'a': 5, 'b': 3} {'b': 2, 'a': 4}
# `[GRM-27]` — `{…}` comprehensions collect into a `Set` or a `Map`; in a map a later key keeps the
# first one's position and takes its value, as a map literal's does; a map whose type the program
# names holds what it says, `str`s here, and one with none owns its text (`[TYP-38]`).

fn main():
    xs = [3, 1, 2]
    squares = {x: x * x for x in xs}
    parities = {x % 2 for x in xs}
    println(squares, parities)
    words = ["b", "a", "b", "a", "a"]
    counts: Map[str, int] = {w: i + 1 for i, w in enumerate(words) if i > 0}
    owned_keys = {w.to_string(): i for i, w in enumerate(words)}
    println(counts, owned_keys)
