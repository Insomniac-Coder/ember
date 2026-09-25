#$ test: compile-pass
#$ rules: LEX-17a, CT-1
# `[LEX-17a]` — `W2015` shows the value the type keeps, in full, so the loss
# is visible (D-324): `0.1234567891234` at `f32` is 0.12345679104328156. A
# literal inside arithmetic or under a minus receives the type as well
# (D-326). A constant's literal is warned once, where it is written, however
# many times the constant is used.

const RATIO: f32 = 0.1234567891234 * 2.0    #$ warning[W2015]: `0.1234567891234` becomes `0.12345679104328156`

fn main():
    negative: f32 = -0.9876543219876    #$ warning[W2015]: `0.9876543219876` becomes `0.9876543283462524`
    println(RATIO, RATIO * RATIO, negative)
