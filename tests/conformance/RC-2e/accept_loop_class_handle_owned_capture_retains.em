#$ test: run-pass
#$ rules: RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, OPT-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 5
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# A loop yield is a borrowed `ref Token`, so materializing `copy` creates one
# owning handle and the `owned fn` environment retains that handle. Neither is
# an implicit per-iteration retain.
# The loop is versioned (`[OPT-2]`): its body, and the retains in it, appear
# once in each copy, and each iteration runs one of them.
class Token:
    value: i32

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        copy: Token = token
        task = owned fn() => copy.value
        println(task())
