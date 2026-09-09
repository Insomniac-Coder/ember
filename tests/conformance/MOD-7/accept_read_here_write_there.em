#$ test: run-pass
#$ rules: MOD-7
## "A field declared `pub(read)` may be **read** wherever a `pub` field could
## be read, but may be **written only from the declaring module**."

from support.health import Health, make, heal

fn main():
    h = make(10)
    println(h.value)
    heal(h, 5)
    println(h.value)
    h.max = 200
    println(h.max)
#$ stdout: 10
#$ 15
#$ 200
