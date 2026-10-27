#$ test: compile-fail
#$ rules: R8
#$ help: extra = String("")
## R8 (c), verdict 72: a string a loop goes through, here by its characters, does not change while the
## loop walks it; the change is refused where it is made (E3020), with the list's message.

fn main():
    text = String("ab")
    for c in text.chars():
        text.push_str("!")    #$ error[E3020]: `text.push_str("!")` (line 10) adds to `text`, whose characters this loop walks (line 9); collect what to add in a text of your own: write `extra = String("")` before the loop, `extra.push_str("!")` in place of this line, and `text.push_str(extra)` after the loop
        println(c)
