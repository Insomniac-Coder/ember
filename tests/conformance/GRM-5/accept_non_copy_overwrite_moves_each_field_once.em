#$ test: run-pass
#$ rules: GRM-5, EXP-2, OWN-2, OWN-5, DRP-2
#$ stdout: rhs
#$ stdout: 10
#$ stdout: 20
#$ stdout: 0
#$ stdout: 1
#$ stdout: 2
#$ stdout: 2
#$ stdout: 1

# The tuple is constructed once before either destination place is changed.
# Each old value is then destroyed before its replacement is stored, and each
# replacement field moves out of the aggregate temporary exactly once.

struct R:
    pub value: i32

    fn drop(mut self):
        println(self.value)

fn replacements() -> (R, R):
    println("rhs")
    return (R(1), R(2))

fn main():
    left = R(10)
    right = R(20)
    left, right = replacements()
    println(0)
    println(left.value)
    println(right.value)
