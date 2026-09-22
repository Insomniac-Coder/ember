#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# A loop over Shared handles yields a borrowed view of the Array element.
# Passing that value to an owned parameter is the explicit RC-2e escape point.
struct Token:
    value: i32

fn consume(owned token: Shared[Token]) -> i32:
    value = token.get()
    return value.value

fn main():
    tokens: Array[Shared[Token]] = Array[Shared[Token]]()
    tokens.push(Shared(Token(7)))
    for token in tokens:
        println(consume(token))
