#$ test: run-pass
#$ rules: EXC-9, EXC-12, EXC-13, EXC-14, TST-15
#$ profiles: debug, development, release
#$ stdout: 6

# `[EXC-13](3)` — publishing another copy of the receiver in the loop defeats
# the stable-receiver proof. The source remains safe because the ordinary
# per-access exclusivity check stays in place.

class Counter:
    value: i32

    fn bump():
        this.value = this.value + 1

    fn print_plus(extra: i32):
        this.value += extra
        println(this.value)

# `report` holds a `Counter` while other code (the printing) runs (`print_plus` changes its
# object, so its call holds it), so a check
# on a `Counter` can fail somewhere and each stays (`[EXC-3]`, ODR-085);
# without it they would all be removed and this loop would show no check.
fn report(ref mut counter: Counter, extra: i32):
    counter.print_plus(extra)

fn main():
    counter = Counter(0)
    alias = counter
    published = counter
    for i in 0..3:
        published = counter
        counter.bump()
    report(alias, published.value)
