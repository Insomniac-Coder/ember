#$ test: run-pass
#$ rules: PHIL-1, DRP-2
#$ stdout: 5
#$ body
#$ 6
#$ end
#$ drop second
#$ drop first
# `[PHIL-1]`: two syntactically identical declarations in one context have identical lifetime. Both
# `Noisy` locals are declared alike; `first` is last used before `body` and `second` after it, but
# neither ends at its last use (a borrow would, a value does not): both live to the end of the block
# and drop there in reverse declaration order (`[DRP-2]`), whatever the program did with them.

struct Noisy:
    name: str

    fn drop(mut self):
        println(f"drop {self.name}")

fn peek(n: ref Noisy) -> int:
    return n.name.len()

fn main():
    first = Noisy("first")
    second = Noisy("second")
    println(peek(first))
    println("body")
    println(peek(second))
    println("end")
