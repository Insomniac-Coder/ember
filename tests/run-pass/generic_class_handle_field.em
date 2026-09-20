#$ test: run-pass
#$ rules: TYP-16, CLS-1, OWN-7, DRP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 42
#$ assert-c: contains(ember_retain)

class Token:
    value: i32

    fn drop(mut self):
        println(self.value)

class Holder[T]:
    value: T

fn main():
    token = Token(42)
    holder = Holder[Token](token)
    println(holder.value.value)
