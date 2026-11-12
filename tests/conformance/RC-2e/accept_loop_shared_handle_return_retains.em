#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CTL-1, CTL-2, TYP-14
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 1
#$ stdout: 7

# Returning a borrowed `shares` loop yield must retain it before the owned source
# Array is released on return from `first`.
struct Token:
    value: i32

fn first(owns tokens: Array[shares mut Token]) -> shares mut Token:
    for token in tokens:
        return shares mut token
    return shares mut Token(0)

fn main():
    tokens: Array[shares mut Token] = Array[shares mut Token]()
    tokens.push(owns shares mut Token(7))
    result = first(owns tokens)
    value = result
    println(value.value)
