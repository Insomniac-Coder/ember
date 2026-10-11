# D-498 probe: one fixed text, `reduce` called <count> times. Input: `<text 0-5> <count>`.
# 0 empty, 1 "a", 2 "aé🙂", 3 English (58 chars, 2 wide), 4 Chinese, 5 accented Latin.
@noinline
fn reduce(text: str, initial: int) -> int:
    total = initial
    for (i, c) in text.char_indices():
        total += i ^ (c as u32 as int)
    return total

fn main():
    texts = ["", "a", "aé🙂", "the quick brown fox, jumps over\tthe lazy dog; héllo wörld\n",
             "敏捷的棕色狐狸跳过了懒狗，然后又跑回了森林里去找它的朋友们一起玩耍和休息一下吧",
             "Ça häßlich: fünf Öfen, zwölf Äpfel, déjà vu, señor, naïve café, crème brûlée"]
    line = input()
    parts = line.split(" ")
    text = texts[parts.next().unwrap().parse[int]().unwrap()]
    n: int = parts.next().unwrap().parse[int]().unwrap()
    acc = 0
    for i in 0..n:
        acc = acc ^ reduce(text, i & 1023)
    println(acc)
