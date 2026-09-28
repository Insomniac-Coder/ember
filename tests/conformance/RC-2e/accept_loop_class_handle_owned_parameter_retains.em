#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, TYP-14, OPT-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 3
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# Passing a loop-yielded handle to an `owned` parameter reads through the
# borrowed reference and retains at the call boundary, never per iteration.
# The loop is versioned (`[OPT-2]`): its body, and the retains in it, appear
# once in each copy, and each iteration runs one of them.
class Token:
    value: i32

fn consume(owned token: Token) -> i32:
    return token.value

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        println(consume(token))
