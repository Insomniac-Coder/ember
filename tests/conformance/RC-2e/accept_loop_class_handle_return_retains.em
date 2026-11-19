#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, TYP-14
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 1
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# Returning a loop-yielded class handle reads through the borrowed reference
# and retains it for the caller before the source Array is cleaned up.
class Token:
    value: i32

fn first(owns tokens: Array[shares Token]) -> shares Token:
    for token in tokens:
        return shares token
    return shares Token(0)

fn main():
    tokens: Array[shares Token] = Array[shares Token]()
    tokens.push(owns shares Token(7))
    println(first(owns tokens).value)
