#$ test: run-pass
#$ rules: R12, R3
#$ stdout: 1
## R12 (rules.md lines 741-748), row 6: the lambda uses `main`'s real `score` and says it changes it. The
## nearest allowed program to reject_ref_mut_in_a_capture_list.

fn main():
    score = 0
    bump = fn() changes score: score += 1
    bump()
    println(score)
