#$ test: run-pass
#$ rules: LEX-4
#$ stdout: 1
## `[LEX-4]` — indentation is spaces, any consistent number of them.

fn main():
  if true:
        println(1)
