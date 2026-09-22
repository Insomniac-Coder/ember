#$ test: run-pass
#$ rules: RC-1, CTL-1, FN-1, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 8
#$ stdout: 7
#$ stdout: 8

# A value-yielding iterator transfers its pair into the loop pattern. The
# leaves become the arm's owned values rather than borrowed Array projections.
class Token:
    value: i32

type Pair = (Token, Token)

struct PairIter:
    done: bool

    fn next(mut self) -> Option[Pair]:
        if self.done:
            return None
        self.done = true
        return Some((Token(7), Token(8)))

fn consume(owned token: Token) -> i32:
    return token.value

fn main():
    for left, right in PairIter(false):
        println(consume(left))
        println(consume(right))
