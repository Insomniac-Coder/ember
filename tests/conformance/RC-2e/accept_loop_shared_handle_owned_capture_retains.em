#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14, RNG-4
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 1
#$ stdout: 7

# Capturing the borrowed `shares` loop yield in an owned closure is an RC-2e
# escape: the closure environment owns a retained copy, not a loop borrow.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
struct Token:
    value: i32

fn main():
    tokens: Array[shares mut Token] = Array[shares mut Token]()
    tokens.push(owns shares mut Token(7))
    for token in tokens:
        kept = shares mut token
        task = owned fn() -> i32:
            return kept.value
        println(task())
