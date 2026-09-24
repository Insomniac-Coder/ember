#$ test: run-pass
#$ rules: SPN-1, LT-1
#$ stdout: hi
# ODR-024 — a borrowed `String` parameter is the caller's string, passed by
# address, so the `str` the coercion makes of it may be returned; the caller
# keeps `s` borrowed while the result lives.

fn borrow_text(text: String) -> str:
    return text

fn main():
    s: String = String()
    s.push_str("hi")
    println(borrow_text(s))
