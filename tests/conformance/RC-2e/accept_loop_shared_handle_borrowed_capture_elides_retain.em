#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, RC-1, RC-2e, CLO-1, CLO-2, CTL-1, CTL-2, TYP-14
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# A normal closure captures the loop's borrowed shares owner by reference. It
# is not an RC-2e escape, so Array insertion is the only strong retain.
struct Token:
    value: i32

fn main():
    tokens: Array[shares mut Token] = Array[shares mut Token]()
    tokens.push(owns shares mut Token(7))
    for token in tokens:
        fn task_fn() -> i32:
            value = token
            return value.value
        task = task_fn
        println(task())
