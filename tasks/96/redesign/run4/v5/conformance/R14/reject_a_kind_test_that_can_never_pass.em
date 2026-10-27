#$ test: compile-fail
#$ rules: R14
## R14: "a test that can never pass, of a type that does not implement the interface or derive from the
## base class (E2037: "`s` is a `Shape`, which can never be a `Path`; remove the test: the `if` line and
## its block")" (the lead's code and message, worklist section 3, U5; section 3's E2037 row, in this
## program's names and lines). Before, step 3's `reject_a_kind_test_between_unrelated_types.em` gave
## E2020, whose meaning no longer lists it. The nearest allowed program is
## `accept_a_kind_test_declares_a_new_name.em`.

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

struct Label:
    text: String

fn radius(s: dyn Shape) -> int:
    if s is Label as l:    #$ error[E2037]: `s` is a `Shape`, which can never be a `Label`; remove the test: the `if` line and its block, lines 23 and 24
        return l.text.len()
    return 0

fn main():
    println(radius(Circle(r=1)))
