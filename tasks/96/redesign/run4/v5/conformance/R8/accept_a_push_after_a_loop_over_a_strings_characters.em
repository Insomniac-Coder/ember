#$ test: run-pass
#$ rules: BRW-18, CTL-2
#$ stdout:
#$ a
#$ b
#$ ab!!
## design rules: R8
## R8 (c), verdict 72: the way out E3020's message gives: what to add is collected in a text of your own
## during the walk and added after the loop. The nearest program to
## `reject_a_push_while_a_loop_walks_a_strings_characters.em` that is allowed.

fn main():
    text = String("ab")
    extra = String("")
    for c in text.chars():
        extra.push_str("!")
        println(c)
    text.push_str(extra)
    println(text)
