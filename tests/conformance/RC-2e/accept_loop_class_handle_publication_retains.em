#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, RNG-4
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 2
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# A yielded handle is borrowed only until an operation gives it a new owner.
# The second retain belongs to `copies.push(token)`, not to loop iteration.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the push's retain, appear once.
class Token:
    value: i32

fn main():
    tokens: Array[Token] = Array[Token]()
    copies: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        copies.push(token)
    println(copies[0].value)
