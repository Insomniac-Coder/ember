#$ test: run-pass
#$ rules: TXT-10
#$ stdout: 14 12 false
#$ 105 14
#$ a|b|c
#$ x|y||z
#$ a|b|c
#$ k v
#$ a - b-c
#$ Héllo, wörld|Héllo, wörld|true|true
#$ Some(8) None Some(12) 3
#$ HéLLo, wörLd|HÉLLO, WÖRLD|héllo, wörld|ababab
#$ false true
#$ Ok(42) Err(ParseError.Invalid) Err(ParseError.Empty) Err(ParseError.Invalid) Err(ParseError.Overflow)
#$ Ok(150.0) Ok(inf) Ok(true) Ok('x') Err(ParseError.Invalid)
#$ Err(ParseError.Invalid) Ok(5) Ok(inf) Ok(nan)
#$ true true true true true false true true true
#$ ǆ SS 3
## `[TXT-10]` - each operation the rule lists, with Unicode-aware `char_count` and case mapping and the
## parse grammar (no white space skipped, `-` only for a signed type, `+` allowed).

fn join(parts: Array[StrView]) -> String:
    out = String()
    for i, p in parts.iter().enumerate():
        if i > 0:
            out.push('|')
        out.push_str(p)
    return out

fn main():
    s = "  Héllo, wörld  "
    t = s.trim()
    println(t.len(), t.char_count(), t.is_empty())
    n = 0
    for _ in t.chars():
        n += 1
    for i, _ in t.char_indices():
        n += i
    for _ in t.bytes():
        n += 1
    println(n, t.as_bytes().len())
    lines: Array[StrView] = []
    for l in "a\nb\r\nc".lines():
        lines.push(owns l)
    println(join(lines))
    parts: Array[StrView] = []
    for w in "x,y,,z".split(","):
        parts.push(owns w)
    println(join(parts))
    words: Array[StrView] = []
    for w in "  a  b c ".split_whitespace():
        words.push(owns w)
    println(join(words))
    match "k=v".split_once("="):
        Some((a, b)):
            println(a, b)
        None:
            println("none")
    (x, y, z) = "a-b-c".partition("-")
    println(x, y, z)
    println(f"{t.trim_start()}|{t.trim_end()}|{t.starts_with("Hé")}|{t.ends_with("d")}")
    println(t.find("wör"), t.find("zz"), t.rfind("l"), t.count("l"))
    println(f"{t.replace("l", "L")}|{t.to_upper()}|{t.to_lower()}|{"ab".repeat(3)}")
    println(t.is_char_boundary(2), "ll" in t)
    println("42".parse[int](), "x".parse[int](), "".parse[int](), "-1".parse[u8](), "300".parse[u8]())
    println("1.5e2".parse[f64](), "INF".parse[f64](), "true".parse[bool](), "x".parse[char](), "xy".parse[char]())
    println(" 4".parse[int](), "+5".parse[int](), "inf".parse[f32](), "nan".parse[f64]())
    println('a'.is_alphabetic(), '٣'.is_digit(), '²'.is_digit(), 'é'.is_alphanumeric(), ' '.is_whitespace(), 'é'.is_ascii_alphabetic(), '5'.is_ascii_digit(), 'a'.is_ascii_alphanumeric(), '\t'.is_ascii_whitespace())
    println("ǅ".to_lower(), "ß".to_upper(), "İ".to_lower().len())
