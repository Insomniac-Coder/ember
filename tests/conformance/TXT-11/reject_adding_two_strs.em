#$ test: compile-fail
#$ rules: TXT-11
#$ profiles: debug
#$ error[E2040]: `str` does not implement `Add`
#$ help: build the text with an f-string

fn main():
    a = "x"
    println(a + "y")
