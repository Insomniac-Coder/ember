#$ test: run-pass
#$ rules: OWN-8, CLO-1
#$ stdout: 1
## design rules: R3, R12
## R3, R12: `fn() owns score` hands the number over to the function value, as it would any other type
## (verdict 84): the function uses it under the same name, and `main` does not use `score` again. The
## machine copies the number either way.

fn main():
    score = 1
    show = fn() owns score: println(score)
    show()
