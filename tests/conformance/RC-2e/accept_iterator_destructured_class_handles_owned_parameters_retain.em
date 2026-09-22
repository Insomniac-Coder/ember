#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 6
#$ stdout: 7
#$ stdout: 8

# The concise tuple pattern also applies to the Span iterator path rather than
# only to Array's direct counted-loop lowering.
class Token:
    value: i32

type Pair = (Token, Token)

fn consume(owned token: Token) -> i32:
    return token.value

fn main():
    pairs: Array[Pair] = Array[Pair]()
    pairs.push((Token(7), Token(8)))
    for left, right in pairs.as_span().iter():
        println(consume(left))
        println(consume(right))
