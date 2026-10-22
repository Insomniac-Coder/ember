class Counter:
    count: int = 0

    fn bump():
        self.count += 1

fn main():
    c = Counter()
    c.bump()
    println(c.count)
