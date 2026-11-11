@must_drop
struct Guard:
    id: int

    fn drop():
        println(f"released {this.id}")

fn main():
    g = Guard(1)
    println(g.id)
