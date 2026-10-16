# D-550 follow-up probe: text in other writing systems. Input: `<mode> <count>`, mode = 3 x script +
# kind. C twin: text_loop_scripts.c, the same strings from the same generator.
#    0: Russian, one long text of 200000 words
#    1: Russian, 10000 strings of 0-3 words
#    2: Russian, 10000 strings of 4-30 words
#    3: Greek, one long text of 200000 words
#    4: Greek, 10000 strings of 0-3 words
#    5: Greek, 10000 strings of 4-30 words
#    6: Arabic, one long text of 200000 words
#    7: Arabic, 10000 strings of 0-3 words
#    8: Arabic, 10000 strings of 4-30 words
#    9: Hindi, one long text of 200000 words
#   10: Hindi, 10000 strings of 0-3 words
#   11: Hindi, 10000 strings of 4-30 words
#   12: Japanese, one long text of 200000 words
#   13: Japanese, 10000 strings of 0-3 words
#   14: Japanese, 10000 strings of 4-30 words
#   15: Korean, one long text of 200000 words
#   16: Korean, 10000 strings of 0-3 words
#   17: Korean, 10000 strings of 4-30 words
#   18: emoji in English, one long text of 200000 words
#   19: emoji in English, 10000 strings of 0-3 words
#   20: emoji in English, 10000 strings of 4-30 words
@noinline
fn reduce(text: str, initial: int) -> int:
    total = initial
    for (i, c) in text.char_indices():
        total += i ^ (c as u32 as int)
    return total

fn pick(seed: int, words: Array[str]) -> str:
    return words[(seed // 65536) % words.len()]

fn main():
    ascii_words = ["the", "quick", "brown", "fox", "jumps", "over", "a", "lazy", "dog", "and", "runs", "far", "away", "from", "home", "today", "while", "it", "rains", "in", "spring", "I"]
    scripts: Array[Array[str]] = [
        ["привет", "мир", "быстрый", "лиса", "собака", "дом", "река", "лес", "друг", "вода"],
        ["γεια", "κόσμος", "γρήγορη", "αλεπού", "σκύλος", "σπίτι", "ποτάμι", "δάσος", "φίλος", "νερό"],
        ["مرحبا", "عالم", "سريع", "ثعلب", "كلب", "بيت", "نهر", "غابة", "صديق", "ماء"],
        ["नमस्ते", "दुनिया", "तेज़", "लोमड़ी", "कुत्ता", "घर", "नदी", "जंगल", "दोस्त", "पानी"],
        ["こんにちは", "世界", "速い", "狐", "犬", "家", "川", "森", "友達", "水"],
        ["안녕하세요", "세계", "빠른", "여우", "개", "집", "강", "숲", "친구", "물"],
        ["😀", "🎉", "🚀", "🐶", "🌲", "🏠", "💧", "🔥", "⭐", "🍕"]
    ]
    spaced = [true, true, true, true, false, true, true]
    ascii_share = [0, 0, 0, 0, 0, 0, 5]
    line = input()
    parts = line.split(" ")
    mode = parts.next().unwrap().parse[int]().unwrap()
    n: int = parts.next().unwrap().parse[int]().unwrap()
    script = mode // 3
    kind = mode % 3
    seed = 12345
    acc = 0
    strings: Array[String] = []
    count_strings = 10000
    if kind == 0:
        count_strings = 1
    for k in 0..count_strings:
        s = String.from("")
        seed = (seed * 1103515245 + 12345) % 2147483648
        words = 200000
        if kind == 1:
            words = (seed // 65536) % 4
        elif kind == 2:
            words = 4 + (seed // 65536) % 27
        for w in 0..words:
            if w > 0 and spaced[script]:
                s.push_str(" ")
            seed = (seed * 1103515245 + 12345) % 2147483648
            which = (seed // 65536) % 10
            seed = (seed * 1103515245 + 12345) % 2147483648
            if which < ascii_share[script]:
                s.push_str(pick(seed, ascii_words))
            else:
                s.push_str(pick(seed, scripts[script]))
        strings.push(s)
    for round in 0..n:
        for s in strings:
            acc = acc ^ reduce(s, round & 1023)
    println(acc)
