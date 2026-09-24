#$ test: compile-fail
#$ rules: TYP-15
#$ error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
# D-199 — `push`, `insert` and `a[i] = v` store an element too, so each checks
# that a view it stores is `static`.

fn main():
    word: String = "hello"
    names = ["ann"]
    names.push(word[1..])
    println(names)
