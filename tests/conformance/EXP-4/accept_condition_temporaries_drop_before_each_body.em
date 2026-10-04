#$ test: run-pass
#$ rules: EXP-4, DRP-3, CTL-4
#$ profiles: debug, release, shipping
#$ stdout: drop 3
#$ stdout: then
#$ stdout: drop 2
#$ stdout: body 1
#$ stdout: drop 1
#$ stdout: body 0
#$ stdout: drop 0
#$ stdout: else
#$ stdout: conditions done
# D-503: a condition's temporaries end before the chosen block runs. If/While
# lowering ended them only after the whole statement, and a while dropped
# only its last condition's. Each while condition destroys its own temporary,
# even on continue, and the final false one before else.

struct Marker:
    pub n: int

    fn drop(mut self):
        println("drop", self.n)

fn positive(marker: Marker) -> bool:
    return marker.n > 0

fn main():
    if positive(Marker(3)):
        println("then")
    remaining = 2
    while positive(Marker(remaining)):
        remaining -= 1
        println("body", remaining)
        continue
    else:
        println("else")
    println("conditions done")
