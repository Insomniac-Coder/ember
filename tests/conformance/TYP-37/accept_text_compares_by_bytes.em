#$ test: run-pass
#$ rules: TYP-37
#$ profiles: debug, release, shipping
#$ stdout: true true false
#$ true true true
#$ true false true
# `str` and `String` compare and order by bytes; a prefix sorts first.

fn main():
    s: String = "apple"
    println(s == "apple", "apple" != "apples", s == "Apple")
    println("apple" < "apples", "Zebra" < "apple", "b" > "abc")
    println(s <= "apple", s >= "b", "" < s)
