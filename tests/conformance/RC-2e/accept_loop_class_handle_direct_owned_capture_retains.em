#$ test: run-pass
#$ rules: RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14, OPT-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 3
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# An owned closure receives its own class-handle copy at the loop-capture
# boundary; the compiler must not preserve the loop's borrowed reference.
# The loop is versioned (`[OPT-2]`): its body, and the retains in it, appear
# once in each copy, and each iteration runs one of them.
class Token:
    value: i32

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        task = owned fn() => token.value
        println(task())
