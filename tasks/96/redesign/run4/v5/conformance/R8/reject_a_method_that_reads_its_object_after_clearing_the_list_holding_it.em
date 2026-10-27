#$ test: compile-fail
#$ rules: BRW-21
#$ help: move line 15 above line 14
## design rules: R8, R4, R5
## R8 (b), (g), verdict 54: `this` counts as a view parameter. `show` only reads its object, and reads it
## after clearing `SHOWN`, which owns it, so `SHOWN[0].show()` is refused (E3031). Verdict 54's own
## example changes `this`; this one only reads it, which the same check covers.

static mut SHOWN: Array[Label] = []

class Label:
    text: String

    fn show():
        SHOWN.clear()
        println(this.text)

fn main():
    SHOWN.push(Label(text=String("hi")))
    SHOWN[0].show()    #$ error[E3031]: `show` clears `SHOWN` and then reads `this.text` (line 15), and `SHOWN[0]` is part of `SHOWN`; in `show`, read `this.text` before `SHOWN.clear()`: move line 15 above line 14
