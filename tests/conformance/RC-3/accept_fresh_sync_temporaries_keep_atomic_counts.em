#$ test: run-pass
#$ rules: RC-3, RC-4, RT-8, THR-1
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain_sync((ember_obj_header*)") == 1
#$ stdout: 42
#$ stdout: cleared

# A new `thread_shared` value's first owner takes no count; one more owner is an atomic count.
class Token:
    value: int

fn main():
    values: Array[thread_shared Token] = Array[thread_shared Token]()
    values.push(owns thread_shared Token(42))
    kept = thread_shared values[0]
    println(kept.value)
    values.clear()
    println("cleared")
