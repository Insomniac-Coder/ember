#$ test: run-pass
#$ rules: RC-3, RC-1, OBJ-2, DSP-3, CTL-2
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 0
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: sum 6
#$ stdout: drop 0
#$ stdout: drop 1
#$ stdout: drop 2
#$ stdout: drop 3
#$ stdout: cleared

# A fresh class owner transfers through its erased temporary into the list.
# No count pair is needed, but the list still owns every object until clear.
interface Value:
    fn value() -> i32

class Token implements Value:
    id: i32

    fn value() -> i32:
        return this.id

    fn drop():
        println("drop", this.id)

fn main():
    tokens: Array[Value] = Array[Value]()
    for i in 0_i32..4_i32:
        tokens.push(owns Token(i))
    total = 0_i32
    for token in tokens:
        total += token.value()
    println("sum", total)
    tokens.clear()
    println("cleared")
