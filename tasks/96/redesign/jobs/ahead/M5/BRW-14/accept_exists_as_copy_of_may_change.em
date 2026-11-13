# Clause BRW-14.7: "The one exception is a name opened `exists as copy of v`, a copy of your own,
# which may be changed (`if scores.get(name) exists as copy of v:` then `v += 1`; the map is
# untouched)."
#$ test: run-pass
#$ rules: BRW-14, GRM-44
#$ stdout: 2
#$ stdout: 1

fn main():
    scores: Map[String, int] = {String("ann"): 1}
    name = String("ann")
    if scores.get(name) exists as copy of v:
        v += 1
        println(v)
    println(scores[name])
