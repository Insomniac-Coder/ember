#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, FN-1, FN-6a, TYP-14, RNG-4
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 1
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# A callable value preserves its `owned` parameter mode, so passing a
# loop-yielded handle through it retains at the indirect call boundary.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
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
