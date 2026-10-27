#$ test: compile-fail
#$ rules: R12, R3
#$ help: bump = fn() changes score: score += 1
## R3 (rules.md line 219) and R12 (lines 741-748): a name not listed in a lambda's capture list is the real
## variable, which the lambda names after `changes` when it changes it; `ref mut` in the capture list is
## refused (E0120). rules.md section 3's line 1032 message, in this program's names. The line would also
## miss `changes score` (E2105), but E0120's fix gives the whole line (R18, verdict 73).

fn main():
    score = 0
    bump = fn() with ref mut score: score += 1    #$ error[E0120]: remove `ref mut score` from the capture list: without a word the function already uses the real `score`; it says it changes it with `changes score`: `bump = fn() changes score: score += 1`
    bump()
    println(score)
