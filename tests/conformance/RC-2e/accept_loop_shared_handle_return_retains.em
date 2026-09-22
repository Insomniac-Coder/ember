#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# Returning a borrowed Shared loop yield must retain it before the owned source
# Array is released on return from `first`.
struct Token:
    value: i32

fn first(owned tokens: Array[Shared[Token]]) -> Shared[Token]:
    for token in tokens:
        return token
    return Shared(Token(0))

fn main():
    tokens: Array[Shared[Token]] = Array[Shared[Token]]()
    tokens.push(Shared(Token(7)))
    result = first(tokens)
    value = result.get()
    println(value.value)
