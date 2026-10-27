#$ test: run-pass
#$ rules: R8, R4, R5
#$ stdout:
#$ hi
#$ 0
## R8 (g), verdict 54: the way out E3031's message gives: `show` reads `this.text` before it clears
## `SHOWN`, so it may be called on `SHOWN[0]`. The nearest program to
## `reject_a_method_that_reads_its_object_after_clearing_the_list_holding_it.em` that is allowed.

static mut SHOWN: Array[Label] = []

class Label:
    text: String

    fn show():
        println(this.text)
        SHOWN.clear()

fn main():
    SHOWN.push(Label(text=String("hi")))
    SHOWN[0].show()
    println(SHOWN.len())
