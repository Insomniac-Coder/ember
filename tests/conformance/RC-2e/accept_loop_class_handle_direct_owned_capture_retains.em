#$ test: run-pass
#$ rules: RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# An owned closure receives its own class-handle copy at the loop-capture
# boundary; the compiler must not preserve the loop's borrowed reference.
class Token:
    value: i32

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        task = owned fn() => token.value
        println(task())
