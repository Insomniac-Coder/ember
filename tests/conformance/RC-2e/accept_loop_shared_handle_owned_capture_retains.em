#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# Capturing the borrowed Shared loop yield in an owned closure is an RC-2e
# escape: the closure environment owns a retained copy, not a loop borrow.
struct Token:
    value: i32

fn main():
    tokens: Array[Shared[Token]] = Array[Shared[Token]]()
    tokens.push(Shared(Token(7)))
    for token in tokens:
        task = owned fn() -> i32:
            value = token.get()
            return value.value
        println(task())
