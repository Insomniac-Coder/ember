#$ test: compile-fail
#$ rules: LT-1, SPN-5
# D-217 — a reborrowed view still borrows what the original points to: an
# item of a local array cannot leave the function through a reborrow of its
# span.

fn first_of_local() -> ref mut int:
    xs = [1, 2, 3]
    span = xs.as_mut_span()
    iterator = span.iter_mut()
    match iterator.next():
        Some(item):
            return item    #$ error[E3060]: `xs` does not live long enough
        None:
            panic("empty")

fn main():
    println(first_of_local())
