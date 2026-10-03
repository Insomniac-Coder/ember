#$ test: run-pass
#$ rules: ERR-4, OWN-6, GRM-15, SPN-3, TYP-36
#$ profiles: debug, release, shipping
#$ stdout: 2 6 7
#$ stdout: none
#$ stdout: 6 7
# Option.take has no Hash requirement. Looking it up must not instantiate
# Option's conditional Hash implementation for an unhashable MutSpan.
# No plain local is declared inside the owned match arm.

fn main():
    values = [5, 7]
    option: Option[MutSpan[int]] = Some(values.as_mut_span())
    taken = option.take()
    match owned taken:
        Some(view):
            view[0] = view[0] + 1
            println(view.len(), view[0], view[1])
        None:
            panic("take lost the mutable view")
    match option:
        Some(_):
            panic("take did not leave None")
        None:
            println("none")
    println(values[0], values[1])
