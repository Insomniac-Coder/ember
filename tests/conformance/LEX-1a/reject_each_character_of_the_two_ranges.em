#$ test: compile-fail
#$ rules: LEX-1a
## `[LEX-1a]` - U+202A to U+202E and U+2066 to U+2069 are all refused, one diagnostic each.

fn main():
    # ‪    #$ error[E0012]: U+202A
    # ‫    #$ error[E0012]: U+202B
    # ‬    #$ error[E0012]: U+202C
    # ‭    #$ error[E0012]: U+202D
    # ‮    #$ error[E0012]: U+202E
    # ⁦    #$ error[E0012]: U+2066
    # ⁧    #$ error[E0012]: U+2067
    # ⁨    #$ error[E0012]: U+2068
    # ⁩    #$ error[E0012]: U+2069
    println("ok")
