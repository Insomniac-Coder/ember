struct Handler:
    run: fn(int) -> int

fn main():
    h: Option[Handler] = None
    if h is None:
        println("none")
