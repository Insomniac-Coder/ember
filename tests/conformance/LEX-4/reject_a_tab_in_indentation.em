#$ test: parse-fail
#$ rules: LEX-4
## `[LEX-4]` — indentation uses spaces; a tab at the start of a logical line
## is `E0002`.

fn main():
	println(1)    #$ error[E0002]: tab used for indentation
