#$ test: run-pass
#$ rules: TYP-13, OBJ-2
#$ profiles: debug, release, shipping
#$ stdout: 8 16
#$ stdout: 7
#$ stdout: none
#$ stdout: dropped
# A class handle cannot be null, so Option[Token] uses null for None.
class Token:
    value: int

    fn drop(mut self):
        println("dropped")

fn main():
    println(mem.size_of[Option[Token]](), mem.size_of[Option[int]]())
    present: Option[Token] = Some(Token(7))
    match present:
        Some(owner):
            println(owner.value)
        None:
            println("unexpected")
    absent: Option[Token] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")
