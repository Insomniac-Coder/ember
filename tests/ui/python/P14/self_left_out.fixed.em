class Counter:
    count: int = 0

    fn bump():
        this.count += 1

fn main():
    c = Counter()
    c.bump()
    println(c.count)
