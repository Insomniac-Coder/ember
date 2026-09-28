#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, TYP-14, OPT-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 8
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7
#$ stdout: 8

# Tuple destructuring yields borrowed handles from the loop's borrowed element.
# Each component receives a retain only when crossing the `owned` call boundary.
# The loop is versioned (`[OPT-2]`): its body, and the retains in it, appear
# once in each copy, and each iteration runs one of them.
class Token:
    value: i32

type Pair = (Token, Token)

fn consume(owned token: Token) -> i32:
    return token.value

fn main():
    pairs: Array[Pair] = Array[Pair]()
    pairs.push((Token(7), Token(8)))
    for left, right in pairs:
        println(consume(left))
        println(consume(right))
