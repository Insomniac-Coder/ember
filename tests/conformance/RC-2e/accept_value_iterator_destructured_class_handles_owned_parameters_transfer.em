#$ test: run-pass
#$ rules: RC-1, CTL-1, FN-1, TYP-14
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 4
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7
#$ stdout: 8

# A value-yielding iterator transfers its pair into the loop pattern. The
# leaves become the arm's owned values rather than borrowed Array projections.
class Token:
    value: i32

type Pair = (Token, Token)

struct PairIter:
    done: bool

    fn next() -> Option[Pair]:
        if this.done:
            return None
        this.done = true
        return Some((Token(7), Token(8)))

fn consume(owns token: Token) -> i32:
    return token.value

fn main():
    for left, right in PairIter(false):
        println(consume(owns left))
        println(consume(owns right))
