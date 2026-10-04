#$ test: run-pass
#$ rules: EXP-1, EXP-4, DRP-2, DRP-3, STR-5
#$ profiles: debug, release, shipping
#$ stdout: 0 2
#$ stdout: drop 2
#$ stdout: drop 1
#$ stdout: capacity done
#$ stdout: true 3
#$ stdout: drop 3
#$ stdout: drop 12
#$ stdout: drop 11
#$ stdout: drop 2
#$ stdout: drop 1
#$ stdout: equality done
# D-488 (the other agent's case). The capacity wrapper must retain an initializer
# producer until the outer println ends. Derived tuple equality must also
# retain the values stored in its own hidden ordered bindings, not just the
# temporaries their initializers made. Eq ignores tens to distinguish sides.

struct Marker:
    pub n: int

    fn drop(mut self):
        println("drop", self.n)

extend Marker implements Eq:
    fn eq(self, other: Marker) -> bool:
        return self.n % 10 == other.n % 10

fn number(marker: Marker) -> int:
    return marker.n

fn main():
    println(String.with_capacity(number(Marker(1))).len(), number(Marker(2)))
    println("capacity done")
    println((Marker(1), Marker(2)) == (Marker(11), Marker(12)), number(Marker(3)))
    println("equality done")
