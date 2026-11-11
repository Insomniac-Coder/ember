#$ test: compile-fail
#$ rules: FN-4, LEX-15
#$ error[E1010]: inside a method its object is `this`
#$ help: write `return this.n * 2`
# `[FN-4]` — `self` anywhere is `E1010` ("inside a method its object is `this`"), whose help gives
# `this`; a `self` in an expression of a method body names the word to write.

struct Counter:
    n: int

    fn get() -> int:
        return self.n * 2

fn main():
    c = Counter(n=2)
    println(c.get())
