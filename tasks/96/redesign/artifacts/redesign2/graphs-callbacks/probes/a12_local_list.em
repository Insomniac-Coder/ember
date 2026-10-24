# A12: the handlers live in a local list that dies before `score` (the case a proof covers).
fn game() -> int:
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): score += 1)
    for h in hs.iter_mut():
        h()
    return score

fn main():
    println(game())
