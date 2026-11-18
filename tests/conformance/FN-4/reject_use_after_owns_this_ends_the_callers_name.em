#$ test: compile-fail
#$ rules: FN-4
# `[FN-4]` — `owns this` takes the object over, so the caller's name ends: a later use is `E3040`.

struct Counter:
    n: u64

    fn finish() owns this -> u64:
        return this.n

fn main():
    c = Counter(n=9)
    println(c.finish())
    println(c.n)    #$ error[E3040]
