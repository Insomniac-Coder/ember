#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# A Shared yielded by an Array loop is borrowed until storing it creates a new
# owner. The retain belongs to `copies.push(token)`, not to loop iteration.
struct Token:
    value: i32

fn main():
    tokens: Array[Shared[Token]] = Array[Shared[Token]]()
    copies: Array[Shared[Token]] = Array[Shared[Token]]()
    tokens.push(Shared(Token(7)))
    for token in tokens:
        copies.push(token)
    value = copies[0].get()
    println(value.value)
