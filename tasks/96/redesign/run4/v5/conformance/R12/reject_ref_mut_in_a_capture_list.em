#$ test: compile-fail
#$ rules: R12, R3
#$ help: bump = fn(): score += 1
## R3 (rules.md lines 236-239: "`ref mut` is not a capture word: a name not listed already is the real
## variable (`fn() ref mut x: x += 1` is refused, E0120 ...)") and R12 (lines 752-753). rules.md section
## 3's line 1085 message, in this program's names, with the whole line to write. The nearest allowed
## program is accept_a_lambda_changes_the_real_variable.

fn main():
    score = 0
    bump = fn() ref mut score: score += 1    #$ error[E0120]: remove `ref mut score`: without a word the function already uses the real `score`, and may change it: `bump = fn(): score += 1`
    bump()
    println(score)
