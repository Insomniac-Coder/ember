#$ test: run-pass
#$ rules: DRP-2, PHIL-5
#$ profiles: debug, release, shipping
#$ stdout:
#$ 3 1
#$ loud 2 dropped
#$ loud 7 dropped
# `[PHIL-5]` — at the end of `main` the process ends. In release and shipping
# builds the list of `Quiet` objects is not freed one by one: freeing memory
# the operating system takes back changes nothing a program can observe. A
# `Loud`, whose `drop` prints, is dropped as ever, alone or held through an
# interface that a class with a `drop` implements. Debug builds free
# everything, for their leak check. The milestone test
# `exit_drops_that_only_free_memory_go_in_release` reads the C.

interface Named:
    fn n(self) -> int

class Quiet implements Named:
    k: int
    items: Array[int]

    fn n(self) -> int:
        return self.k

class Loud implements Named:
    k: int

    fn n(self) -> int:
        return self.k

    fn drop(mut self):
        println(f"loud {self.k} dropped")

fn main():
    quiet: Array[Quiet] = []
    for i in 0..3:
        quiet.push(Quiet(i, []))
    named: Array[Named] = []
    named.push(Loud(7))
    loud = Loud(2)
    println(f"{len(quiet)} {len(named)}")
