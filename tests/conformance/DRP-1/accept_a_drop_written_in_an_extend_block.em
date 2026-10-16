#$ test: run-pass
#$ rules: DRP-1, STR-3, DRP-2
#$ profiles: debug, release
#$ stdout: first second
#$ drop second
#$ drop first
# `[DRP-1]`, `[STR-3]` (D-661) — a `drop` written in an `extend` block is the type's destructor,
# as one in its body is: it runs once per value, locals in reverse declaration order. It never
# ran before: only a generic extension's `drop` gave its type one.

struct Noisy:
    name: str

extend Noisy:
    fn drop(mut self):
        println("drop", self.name)

fn plain():
    first = Noisy("first")
    second = Noisy("second")
    println(first.name, second.name)

fn main():
    plain()
