#$ test: run-pass
#$ rules: STD-13, PHIL-14
#$ stdout: k = v 2 HI true
#$ ['ann', 'bob'] 1
# `[STD-13]` — Ember's names are `snake_case` words (`starts_with`, `to_upper`, `push`); a Python name
# Ember keeps (`partition`, `count`) keeps Python's meaning.

fn main():
    left, sep, right = "k=v".partition("=")
    println(left, sep, right, "a-b-a".count("a"), " Hi ".trim().to_upper(), "Hi".starts_with("H"))
    names = ["ann"]
    names.push("bob")
    println(names, "a b".lines().count())
