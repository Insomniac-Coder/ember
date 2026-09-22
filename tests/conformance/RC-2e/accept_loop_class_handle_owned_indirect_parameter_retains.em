#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, FN-1, FN-6a, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# A callable value preserves its `owned` parameter mode, so passing a
# loop-yielded handle through it retains at the indirect call boundary.
class Token:
    value: i32

fn consume(owned token: Token) -> i32:
    return token.value

fn main():
    operation: fn(owned Token) -> i32 = consume
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        println(operation(token))
