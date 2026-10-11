# D-498/D-550 probe: random text. Input: `<mode> <count>`. 0 English-like 1.1 MB, 1 Chinese-like,
# 2 10000 short strings of 0-3 mixed words, 3 the same of ASCII words, 4 mixed without empty strings,
# 5 10000 strings of 4-30 mixed words, 6 the same of ASCII words. C twin: text_loop_varied_text.c.
@noinline
fn reduce(text: str, initial: int) -> int:
    total = initial
    for (i, c) in text.char_indices():
        total += i ^ (c as u32 as int)
    return total

fn pick(seed: int, words: Array[str]) -> str:
    return words[(seed // 65536) % words.len()]

fn main():
    ascii_words = ["the", "quick", "brown", "fox", "jumps", "over", "a", "lazy", "dog", "and", "runs",
                   "far", "away", "from", "home", "today", "while", "it", "rains", "in", "spring", "I"]
    latin_words = ["héllo", "wörld", "café", "naïve", "señor", "déjà", "Öfen", "zwölf", "crème", "brûlée"]
    cjk_words = ["敏捷", "棕色", "狐狸", "跳过", "懒狗", "森林", "朋友", "休息", "一起", "玩耍"]
    line = input()
    parts = line.split(" ")
    mode = parts.next().unwrap().parse[int]().unwrap()
    n: int = parts.next().unwrap().parse[int]().unwrap()
    seed = 12345
    acc = 0
    if mode >= 2:
        # Many short strings of 0-3 words: mostly ASCII, some accented, some Chinese.
        strings: Array[String] = []
        for k in 0..10000:
            s = String.from("")
            seed = (seed * 1103515245 + 12345) % 2147483648
            count = (seed // 65536) % 4
            if mode >= 3:
                count = 1 + (seed // 65536) % 3
            if mode >= 5:
                count = 4 + (seed // 65536) % 27
            for w in 0..count:
                if w > 0:
                    s.push_str(" ")
                seed = (seed * 1103515245 + 12345) % 2147483648
                kind = (seed // 65536) % 10
                seed = (seed * 1103515245 + 12345) % 2147483648
                if kind < 7 or mode == 3 or mode == 6:
                    s.push_str(pick(seed, ascii_words))
                elif kind < 9:
                    s.push_str(pick(seed, latin_words))
                else:
                    s.push_str(pick(seed, cjk_words))
            strings.push(s)
        for round in 0..n:
            for s in strings:
                acc = acc ^ reduce(s, round & 1023)
    else:
        # One long text of 200000 words: English-like (mode 0) or Chinese-like (mode 1).
        text = String.from("")
        for k in 0..200000:
            seed = (seed * 1103515245 + 12345) % 2147483648
            kind = (seed // 65536) % 100
            seed = (seed * 1103515245 + 12345) % 2147483648
            if mode == 0:
                if kind < 95:
                    text.push_str(pick(seed, ascii_words))
                else:
                    text.push_str(pick(seed, latin_words))
                text.push_str(" ")
            else:
                if kind < 90:
                    text.push_str(pick(seed, cjk_words))
                else:
                    text.push_str(pick(seed, ascii_words))
                    text.push_str(" ")
        for round in 0..n:
            acc = acc ^ reduce(text, round & 1023)
    println(acc)
