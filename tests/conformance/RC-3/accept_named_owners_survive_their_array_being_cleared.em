#$ test: run-pass
#$ rules: RC-3, RC-1, DRP-2, WK-2, WK-3
#$ profiles: debug, development, release
#$ stdout: before
#$ stdout: live 7
#$ stdout: after
#$ stdout: drop 7

# The optimization must not consume a named owner's last use. Clearing its
# copied list keeps that owner, and `weakref`.upgrade remains successful.
class Token:
    id: i32

    fn drop():
        println("drop", this.id)

fn main():
    held = Token(7)
    weak = weakref held
    tokens: Array[shares Token] = Array[shares Token]()
    tokens.push(owns shares held)
    println("before")
    tokens.clear()
    if weak exists as value:
        println("live", value.id)
    else:
        println("expired")
    println("after")
