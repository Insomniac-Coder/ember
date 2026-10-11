@must_drop
struct Guard:
    id: int

    fn drop(mut self):
        println(f"released {self.id}")

fn main():
    g = Guard(1)
    println(g.id)
