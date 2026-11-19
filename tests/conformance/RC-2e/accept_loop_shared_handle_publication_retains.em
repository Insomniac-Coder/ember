#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CTL-1, CTL-2, RNG-4, TYP-14
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 1
#$ stdout: 7

# A `shares` yielded by an Array loop is borrowed until storing it creates a new
# owner. The retain belongs to `copies.push(token)`, not to loop iteration.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the push's retain, appear once.
struct Token:
    value: i32

fn main():
    tokens: Array[shares mut Token] = Array[shares mut Token]()
    copies: Array[shares mut Token] = Array[shares mut Token]()
    tokens.push(owns shares mut Token(7))
    for token in tokens:
        copies.push(owns (shares mut token))
    println(copies[0].value)
