#$ test: run-pass
#$ rules: RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14, RNG-4
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 1
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# An owned closure receives its own class-handle copy at the loop-capture
# boundary; the compiler must not preserve the loop's borrowed reference.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
class Token:
    value: i32

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        task = owned fn() => token.value
        println(task())
