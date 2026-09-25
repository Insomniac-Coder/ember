#$ test: run-pass
#$ rules: TYP-23, STD-17
#$ stdout: {'even': []} {'x': None} {'m': {}}
#$ [5] 0
# D-297 — in a call to a generic method, an argument whose parameter type no
# type parameter reaches is checked against that type, so `[]`, `None` and
# `{}` take their type from it: `m[k] = []` stores through `index_set`, whose
# value parameter is the map's `V`. They were E2060, checked with no type.

fn main():
    groups: Map[String, Array[int]] = {}
    groups["even"] = []
    best: Map[String, Option[int]] = {}
    best["x"] = None
    nested: Map[String, Map[String, int]] = {}
    nested["m"] = {}
    println(groups, best, nested)
    groups["even"].push(5)
    counts: Map[String, int] = {}
    println(groups["even"], counts.get_or("zz", 0))
