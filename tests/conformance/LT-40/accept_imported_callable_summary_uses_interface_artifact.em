#$ test: run-pass
#$ rules: LT-22, LT-35, LT-40, MIR-REG-1, VERIFY-3
#$ assert-c: !contains("callable-regions")
#$ stdout: 22

from support.views import bundle, right_value

fn main():
    left: Array[i32] = Array[i32]()
    left.push(11)
    right: Array[i32] = Array[i32]()
    right.push(22)

    pair = bundle(left.as_span(), right.as_span())
    # The imported summary must retain only `right` through pair.right.
    left.push(33)
    println(right_value(pair))
