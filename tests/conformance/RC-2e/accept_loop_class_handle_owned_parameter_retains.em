#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# Passing a loop-yielded handle to an `owned` parameter reads through the
# borrowed reference and retains at the call boundary, never per iteration.
class Token:
    value: i32

fn consume(owned token: Token) -> i32:
    return token.value

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        println(consume(token))
