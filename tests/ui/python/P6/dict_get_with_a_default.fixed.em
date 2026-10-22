fn main():
    counts: Map[String, int] = {"a": 1}
    println(counts.get_or("b", 0))
