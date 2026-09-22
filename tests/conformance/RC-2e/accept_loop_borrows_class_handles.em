#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 1
#$ stdout: 7

# Iteration borrows each class handle from the Array; merely reading it in the
# body must not create a temporary owning handle.
class Token:
    value: i32

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        println(token.value)
