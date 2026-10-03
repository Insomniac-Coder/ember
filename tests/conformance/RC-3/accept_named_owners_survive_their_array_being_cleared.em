#$ test: run-pass
#$ rules: RC-3, RC-1, DRP-2, WK-2, WK-3
#$ profiles: debug, release, shipping
#$ stdout: before
#$ stdout: live 7
#$ stdout: after
#$ stdout: drop 7

# The optimization must not consume a named owner's last use. Clearing its
# copied list keeps that owner, and Weak.upgrade remains successful.
class Token:
    id: i32

    fn drop(mut self):
        println("drop", self.id)

fn main():
    held = Token(7)
    weak = Weak(held)
    tokens: Array[Token] = Array[Token]()
    tokens.push(held)
    println("before")
    tokens.clear()
    match weak.upgrade():
        Some(value) => println("live", value.id)
        None => println("expired")
    println("after")
