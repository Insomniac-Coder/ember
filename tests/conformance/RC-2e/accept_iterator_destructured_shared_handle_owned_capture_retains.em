#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 3
#$ stdout: 7

# The borrowed Span-iterator path has the same destructured Shared capture
# boundary as direct Array iteration: only tuple construction, Array insertion,
# and the owned closure environment retain the owner.
struct Token:
    value: i32

type Pair = (Shared[Token], i32)

fn main():
    pairs: Array[Pair] = Array[Pair]()
    pairs.push((Shared(Token(7)), 0))
    for token, _ in pairs.as_span().iter():
        task = owned fn() -> i32:
            value = token.get()
            return value.value
        println(task())
