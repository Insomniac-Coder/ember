# A12: two closures sharing one written variable (phase3-break-D2.md N2).
fn make_pair() -> (fn(), fn() -> int):
    count = 0
    fn inc():
        count += 1
    fn get() -> int:
        return count
    return (inc, get)

fn main():
    (inc, get) = make_pair()
    inc()
    inc()
    println(get())
