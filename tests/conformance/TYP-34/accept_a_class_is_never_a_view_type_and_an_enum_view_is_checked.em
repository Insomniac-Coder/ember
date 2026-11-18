#$ test: run-pass
#$ rules: TYP-34
#$ stdout: hello
#$ 5
## `[TYP-34]` - a class is never a view type (a view field of a class says what it points into); an enum
## with a view payload is a view type that may live as long as the text it names.

class Label:
    text: StrView

enum Pick:
    Name(StrView)
    Nothing

fn show(p: Pick):
    match p:
        Name(s):
            println(s)
        Nothing:
            println("none")

fn main():
    word = "hello"
    l = Label("hello")
    show(Pick.Name(word))
    println(l.text.len())
