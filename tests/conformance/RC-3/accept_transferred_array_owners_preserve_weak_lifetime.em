#$ test: run-pass
#$ rules: RC-3, RC-1, DRP-2, WK-11, WK-12, WK-14
#$ profiles: debug, release, shipping
#$ stdout: live 7
#$ stdout: drop 7
#$ stdout: cleared
#$ stdout: expired

# A fresh temporary transfers to the Array. Weak observations must still see
# a live object until clear, then an expired object after its final owner ends.
class Token:
    id: i32

    fn drop(mut self):
        println("drop", self.id)

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    weak = Weak(tokens[0])
    match weak.upgrade():
        Some(value) => println("live", value.id)
        None => println("unexpected expiration")
    tokens.clear()
    println("cleared")
    match weak.upgrade():
        Some(value) => println("unexpected owner", value.id)
        None => println("expired")
