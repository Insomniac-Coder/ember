#$ test: compile-fail
#$ rules: R3
#$ help: to make your own copy write `String(w)`
## R3: `copy of` is refused before a value of a view type (E2116): a `StrView` is not the text, so the
## message gives the call that makes a `String` of your own (AN-07's message).

fn main():
    line = String(" hi ")
    w = line.trim()
    c = copy of w              #$ error[E2116]: `w` is a view of `line`'s text
    println(c)
