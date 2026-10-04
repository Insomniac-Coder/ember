#$ test: compile-fail
#$ rules: CTL-10, EXP-6
#$ profiles: debug, release, shipping
# D-490: a hoisted name an arm moved is moved after the branch on that path;
# reading it there is E3040 at the read, as for any variable.

struct Marker:
    pub n: int

fn consume(owned marker: Marker) -> int:
    return marker.n

fn main():
    flag = true
    if flag:
        kept = Marker(1)
        println(consume(kept))
    else:
        kept = Marker(2)
    println(kept.n)  #$ error[E3040]: `kept` may have been moved out of
