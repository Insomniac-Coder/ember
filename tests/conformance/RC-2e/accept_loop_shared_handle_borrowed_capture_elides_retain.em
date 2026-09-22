#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 1
#$ stdout: 7

# A normal closure captures the loop's borrowed Shared handle by reference. It
# is not an RC-2e escape, so Array insertion is the only strong retain.
struct Token:
    value: i32

fn main():
    tokens: Array[Shared[Token]] = Array[Shared[Token]]()
    tokens.push(Shared(Token(7)))
    for token in tokens:
        task = fn() -> i32:
            value = token.get()
            return value.value
        println(task())
