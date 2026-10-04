#$ test: run-pass
#$ rules: RC-3, RC-4, RT-8, THR-1
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain_sync((ember_obj_header*)") == 1
#$ stdout: 42
#$ stdout: cleared

# This conservative slice transfers only known non-Sync temporaries. Sync
# copies retain their atomic ordering and count operations.
@sync
class Token:
    value: int

fn main():
    values: Array[Token] = Array[Token]()
    values.push(Token(42))
    println(values[0].value)
    values.clear()
    println("cleared")
