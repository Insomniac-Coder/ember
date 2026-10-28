#$ test: run-pass
#$ rules: R13, R3
#$ stdout: 4
#$ 3
## R13, R3: `exists as copy of v` is a copy of your own: it may change (verdict 67), and the map's
## value stays as it was (the nearest program allowed for `R2/reject_a_change_of_a_plain_opened_name`).

fn main():
    scores: Map[String, int] = Map()
    scores[String("ann")] = 3
    if scores.get("ann") exists as copy of v:
        v += 1
        println(v)
    if scores.get("ann") exists as w:
        println(w)
