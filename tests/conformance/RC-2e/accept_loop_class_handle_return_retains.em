#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# Returning a loop-yielded class handle reads through the borrowed reference
# and retains it for the caller before the source Array is cleaned up.
class Token:
    value: i32

fn first(owned tokens: Array[Token]) -> Token:
    for token in tokens:
        return token
    return Token(0)

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    println(first(tokens).value)
