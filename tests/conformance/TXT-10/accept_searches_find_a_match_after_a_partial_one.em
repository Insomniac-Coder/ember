#$ test: run-pass
#$ rules: TXT-10, STD-8b
#$ profiles: debug, release, shipping
#$ stdout: Some(1) Some(3) None Some(0) Some(3)
#$ 2 1 3 4
#$ xZZ héllo! true false true
# `[TXT-10]` — a search finds its needle after a place where only part of it
# matched; `count` counts matches that do not overlap, an empty needle
# matching between characters; `replace` replaces every match. The search
# finds the needle's first byte, then compares the rest (ADR-106).

fn main():
    println("aaab".find("aab"), "abcabd".find("abd"), "a".find("abc"), "ab".find(""), "hé!".find("!"))
    println("aaaa".count("aa"), "abab".count("aba"), "héé".count("é") + 1, "abc".count(""))
    println("xyxyx".replace("yx", "Z"), "hello!".replace("e", "é"), "él" in "héllo", "ab" in "ba", 'é' in "héllo")
