#$ test: run-pass
#$ rules: R2
#$ stdout: 3
## R2: a name declared with `=` stands for its own place; if its value is a view, `=` makes the name
## view something else (`cur = p`), checked by R8 against its new target. The "best so far" walk,
## which today's `[BRW-1]` refuses (`reject_a_reference_is_not_re_pointed`).

struct Reading:
    celsius: int

fn main():
    readings = [Reading(celsius=1), Reading(celsius=3), Reading(celsius=2)]
    hottest = readings[0]
    for r in readings:
        if r.celsius > hottest.celsius:
            hottest = r
    println(hottest.celsius)
