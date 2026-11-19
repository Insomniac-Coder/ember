#$ test: compile-fail
#$ rules: FN-5, BRW-3
# A shared reference made by a later default may inspect a reserved mutable
# receiver while it is evaluated, but cannot remain live when that receiver's
# mutable borrow activates for the call: `collide` changes its object (`this.total.push_str("!")`).

struct Counter:
    total: String

extend Counter:
    fn collide(ref alias: String = ref this.total):
        this.total.push_str("!")
        println(alias)

fn main():
    counter = Counter(total="4".to_string())
    counter.collide() #$ error[E3021]: borrowed
