#$ test: compile-fail
#$ rules: HEAP-3, HEAP-5, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping

# A normal closure preserves the loop yield as `ref Shared[Token]`. Receiver
# auto-dereference finds `get_mut`, but the inherited shared reference must
# still prevent this mutable access.
struct Token:
    value: i32

fn main():
    tokens: Array[Shared[Token]] = Array[Shared[Token]]()
    tokens.push(Shared(Token(7)))
    for token in tokens:
        task = fn():
            value = token.get_mut() #$ error[E3021]: cannot write through a shared reference
            value.value = 8
        task()
