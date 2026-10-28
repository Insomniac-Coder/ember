#$ test: run-pass
#$ rules: GRM-42, CLO-2
#$ stdout: 1
## design rules: R12, R3
## Row 6, R12 (rules.md lines 752-753), R3 (lines 232-234): a name not in the capture list is the real
## variable, and the lambda changes `main`'s real `score` with no word (verdict 86). The nearest allowed
## program to reject_ref_mut_in_a_capture_list.

fn main():
    score = 0
    bump = fn(): score += 1
    bump()
    println(score)
