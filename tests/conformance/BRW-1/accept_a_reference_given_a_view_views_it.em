#$ test: run-pass
#$ rules: BRW-1, BRW-13
#$ stdout: 3.0
# `[BRW-13]` — `hottest = r` on a view name given another view views that instead, checked by
# `[BRW-1]` against its new target (it was D-823's refused rebinding while a reference was never
# pointed elsewhere, ADR-010).

struct Reading:
    celsius: float

fn main():
    readings = [Reading(celsius=1.0), Reading(celsius=3.0)]
    hottest = ref readings[0]
    for r in readings:
        if r.celsius > hottest.celsius:
            hottest = r
    println(hottest.celsius)
