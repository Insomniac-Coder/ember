#$ test: compile-fail
#$ rules: DIA-11, EFF-11, EFF-13
#$ help: hoist the handle into a local the code does not reassign
## `[DIA-11]` — a per-access exclusivity check the analysis did not prove away is
## `not_proven_by_analysis` (`[EFF-11]`): two receivers that may be one object, in a loop, while the
## caller reads the field. The suggestion is a restructuring the analysis can prove.

class Bag:
    items: Array[int]

    fn add(mut self, x: int):
        self.items.push(x)

@static_safe
fn fill(mut b: Bag, mut other: Bag):
    for i in 0..3:
        b.add(i)  #$ error[E4030]: the check's reason is `not_proven_by_analysis`
        other.add(i)  #$ error[E4030]: the check's reason is `not_proven_by_analysis`

fn main():
    b = Bag([])
    c = b
    fill(b, c)
    for x in c.items:
        println(x)
