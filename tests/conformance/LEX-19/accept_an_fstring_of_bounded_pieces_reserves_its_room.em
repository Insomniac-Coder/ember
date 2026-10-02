#$ test: run-pass
#$ rules: LEX-19
#$ profiles: debug, release
#$ assert-c: contains("vec_reserve_more(")
#$ stdout: key-9223372036854775808|18446744073709551615|false|é|-128|65535
#$ stdout: key7 key123456
# `[LEX-19]` — an f-string whose pieces all have a longest text (literal
# text, an integer, a `bool`, a `char`) asks for all that room before it
# appends them, so even the longest of each fits where it was asked for.

fn main():
    a = i64.MIN
    b = u64.MAX
    c = false
    d = 'é'
    e: i8 = -128
    f: u16 = 65535
    println(f"key{a}|{b}|{c}|{d}|{e}|{f}")
    short = f"key{7}"
    println(short, f"key{123456}")
