#$ test: run-pass
#$ rules: R12
#$ stdout: 0
## R12 (rules.md lines 512, 537-542): a lambda that only reads the real `score` changes nothing outside
## itself, so it is a plain `fn()` and fits a list of `fn()`. The nearest allowed program to
## reject_a_lambda_that_names_a_variable_it_never_changes.

fn main():
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): println(score))
    for ref mut h in hs:
        h()
