#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# A yielded handle is borrowed only until an operation gives it a new owner.
# The second retain belongs to `copies.push(token)`, not to loop iteration.
class Token:
    value: i32

fn main():
    tokens: Array[Token] = Array[Token]()
    copies: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        copies.push(token)
    println(copies[0].value)
