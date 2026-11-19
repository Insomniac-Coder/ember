#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CTL-1, CTL-2, TYP-14, RNG-4
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 1
#$ stdout: 7

# A loop over shares owners yields a borrowed view of the Array element.
# Passing that value to an owned parameter is the explicit RC-2e escape point.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
struct Token:
    value: i32

fn consume(owns token: shares mut Token) -> i32:
    value = token
    return value.value

fn main():
    tokens: Array[shares mut Token] = Array[shares mut Token]()
    tokens.push(owns shares mut Token(7))
    for token in tokens:
        println(consume(owns (shares mut token)))
