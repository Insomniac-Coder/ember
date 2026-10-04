#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, TYP-14, RNG-4
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 4
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7
#$ stdout: 8

# Tuple destructuring yields borrowed handles from the loop's borrowed element.
# Each component receives a retain only when crossing the `owned` call boundary.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
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
