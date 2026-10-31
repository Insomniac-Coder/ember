#$ test: compile-fail
#$ rules: FN-5, BRW-3
# A shared reference made by a later default may inspect a reserved mutable
# receiver while it is evaluated, but cannot remain live when that receiver's
# mutable borrow activates for the call: `collide` changes its object (`this.total += 1`).

struct Counter:
    total: int

extend Counter:
    fn collide(alias: ref int = ref this.total):
        this.total += 1
        println(alias)

fn main():
    counter = Counter(total=4)
    counter.collide() #$ error[E3021]: borrowed
