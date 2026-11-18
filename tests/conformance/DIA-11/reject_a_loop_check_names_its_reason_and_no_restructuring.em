#$ test: compile-fail
#$ rules: DIA-11, EFF-11, EFF-13
#$ help: remove `@static_safe`, or change the data structure
#$ not-help: hoist the handle
## `[DIA-11]` — shape S1 (`E4030`) names the check's reason (`[EFF-11]`). A loop's one exclusivity
## check on a shared object is `inherent_to_mechanism`: no restructuring of the code removes it, so
## the suggestion is to remove the contract or change the data structure, never to restructure.

class Counter:
    value: i32

    items: Array[int] = []

    fn bump():
        this.items.push(owns 1)

    fn held(other: shares mut Counter):
        view = this.items[..]
        other.bump()
        println(view.len())

fn observe(c: Counter):
    println(c.value)

@no_alias_checks
fn run(n: int) -> i32:
    counter = Counter(0)
    observe(counter)
    for i in 0..n:
        counter.bump()  #$ error[E4030]: the check's reason is `inherent_to_mechanism`
    return counter.value

fn main():
    println(run(3))
    k = Counter(0)
    k.held(shares mut k)
