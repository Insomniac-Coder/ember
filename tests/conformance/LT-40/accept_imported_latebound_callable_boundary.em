#$ test: run-pass
#$ rules: FN-6b, LT-7, LT-10, LT-40, TST-20, TST-21, MIR-REG-1, VERIFY-3
#$ assert-c: !contains("callable-regions")
#$ stdout: 33

from support.views import latebound_value

fn first(view: Span[i32]) -> i32:
    return view[0]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(33)
    println(latebound_value(first, values.as_span()))
