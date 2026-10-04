#$ test: run-pass
#$ rules: CTL-10, OWN-1, EXP-6, DRP-2
#$ profiles: debug, release, shipping
#$ stdout: 3
#$ stdout: drop 3
#$ stdout: kept 7
#$ stdout: done
#$ stdout: drop 7
# D-490: a name declared in every completing arm is one variable with the name
# hoisted beside the branch (`[CTL-10]`), so an arm may move it: it is then
# moved after the branch, an error only if read. A copy into the hoisted name
# at the arm's end read the moved value again (E3040) and left the names the
# arm did read reported as never read (L1001). `kept` is read after the match;
# `moved` and `wrapped` are consumed inside the arm.

struct Marker:
    pub n: int

    fn drop(mut self):
        println("drop", self.n)

fn take(marker: Marker) -> int:
    return marker.n

fn main():
    value: Option[Marker] = Some(Marker(3))
    match owned value:
        Some(inner):
            moved = inner
            wrapped = Some(moved)
            kept = Marker(7)
            match owned wrapped:
                Some(last):
                    println(take(last))
                None:
                    panic("impossible")
        None:
            panic("impossible")
    println("kept", kept.n)
    println("done")
