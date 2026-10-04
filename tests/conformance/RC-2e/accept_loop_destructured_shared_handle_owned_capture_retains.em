#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14, RNG-4
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# A tuple pattern still yields a borrowed Shared handle. Capturing that leaf in
# an owned closure must copy it into the closure environment. The two retains
# are Array insertion and that escape boundary; none is at the loop yield, and
# the tuple takes its fresh `Shared` by a move (`[RC-2d]`).
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
struct Token:
    value: i32

type Pair = (Shared[Token], i32)

fn main():
    pairs: Array[Pair] = Array[Pair]()
    pairs.push((Shared(Token(7)), 0))
    for token, _ in pairs:
        task = owned fn() -> i32:
            value = token.get()
            return value.value
        println(task())
