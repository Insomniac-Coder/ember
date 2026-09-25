#$ test: compile-fail
#$ rules: STD-20
# ODR-039 — an operator method's other operand has the receiver's type (a
# shift amount and an exponent may be any integer type); a shift has no
# saturating form; a scalar type has only the constants `[STD-20]` names.

fn main():
    n: i32 = 5
    println(n.checked_add(1.5))    #$ error[E2020]: expected `i32`, found `a float`
    println(n.saturating_shl(1))    #$ error[E1010]: `i32` has no method named `saturating_shl`
    println(n.checked_shl(2.0))    #$ error[E2020]: `checked_shl` takes an integer, not `f64`
    println(i32.LARGEST)    #$ error[E1010]: `i32` has no constant `LARGEST`
