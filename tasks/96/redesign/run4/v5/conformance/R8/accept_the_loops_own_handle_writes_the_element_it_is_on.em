#$ test: run-pass
#$ rules: BRW-18, CTL-2
#$ stdout: [5, 5, 5]
## design rules: R8, R2
## R8 (c), verdict 95: the loop's own handle may change the element it is on, as Rust's `iter_mut()`:
## `for ref mut x in xs: x = 5`. The nearest program to
## `reject_a_write_of_an_element_while_a_for_loop_walks_the_list.em` that is allowed, with the lines its
## message gives.

fn main():
    xs = [1, 2, 3]
    for ref mut x in xs:
        x = 5
    println(xs)
