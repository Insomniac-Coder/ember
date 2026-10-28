#$ test: run-pass
#$ rules: LT-45
#$ stdout: 3
## design rules: R7, R6
## R7: a struct that holds a view may be returned when what it views outlives the call: here the
## caller's list (R6: a function's result points into its view arguments). The nearest program to
## `reject_a_struct_holding_a_view_returned_past_its_source.em` that is allowed.

struct Holder:
    s: Span[int]

fn make(xs: Array[int]) -> Holder:
    return Holder(s=xs.as_span())

fn main():
    nums = [1, 2, 3]
    h = make(nums)
    println(h.s.len())
