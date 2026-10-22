fn best(scores: Array[int]) -> int:
    return scores.iter().max().unwrap()

fn main():
    println(best([3, 9, 4]))
