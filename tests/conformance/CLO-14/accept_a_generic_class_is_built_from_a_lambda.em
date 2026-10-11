#$ test: run-pass
#$ rules: CLO-14, CLS-10, CLO-2
#$ profiles: debug, release
#$ stdout: 10
#$ stdout: 7
# `[CLO-14]` (D-546) — a generic class whose type argument is a closure's type
# is built from a lambda: the argument is read once to infer the type argument
# and checked again against the instance, and both are the one closure. One
# that captures by move holds what it took; one that captures nothing holds no
# borrow. One that borrows a local is `[TYP-15]`'s `E3063` (a class field holds
# only `static` views), even a local holding a string literal: the capture is
# a reference to the local.

class Holder[F: fn(int)]:
    f: F

    fn init(mut self, owned f: F):
        self.f = f

fn main():
    total = "abcdefghij".to_string()
    counted = Holder(owned fn(v: int) => println(total.len() * v))
    counted.f(1)
    plain = Holder(fn(v: int) => println(v + 5))
    plain.f(2)
