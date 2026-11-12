@must_drop
struct Guard:
    id: int

    fn drop():
        println(f"released {this.id}")

fn make() -> Guard:
    g = Guard(1)
    return g

fn main():
    _g = make()
