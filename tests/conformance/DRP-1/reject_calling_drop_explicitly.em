#$ test: compile-fail
#$ rules: DRP-1, OWN-2
# "`fn drop(mut self)` is invoked exactly once per value at the end of its
# life. It may not be called explicitly (`E3070`)."
#
# "Exactly once" is the whole rule. An explicit call does not *replace* the one
# at the end of the life — the value is still live afterwards and is still
# dropped at scope end — so the destructor runs twice and whatever it owns is
# freed twice. This compiled, and with a `Array[i32]` field it was a double
# free whose second run also read the buffer after freeing it, with no `unsafe`
# anywhere in the program.

struct R:
    pub v: Array[i32]

    fn drop(mut self):
        println(self.v[0])

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    r = R(a)
    r.drop()               #$ error[E3070]: `R`'s `drop` may not be called explicitly
    println(2)
