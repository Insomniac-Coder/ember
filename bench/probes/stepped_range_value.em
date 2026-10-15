## A stepped `range` held as a value and iterated, against the C loop doing the same (with
## Ember's floor `%` and its overflow check): D-593.

fn main():
    n = 60000000
    stepped = 0
    counting_down = range(n * 3, 0, -3)
    for v in counting_down:
        stepped += v % 11
    println(stepped)
