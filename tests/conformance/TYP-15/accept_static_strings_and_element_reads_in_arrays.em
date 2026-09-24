#$ test: run-pass
#$ rules: TYP-15
#$ profiles: debug, release, shipping
#$ stdout: ['zed', 'al', 'bob', 'cy']
#$ ['bo', 'cy'] ['ann', 'bo'] ['bo', 'cy']
# `[TYP-15]` — `names = ["ann", "bob"]` is an `Array[str]` of static strings.
# A view read back out of such an `Array` is one of its elements, so it may be
# stored in another.

fn main():
    names = ["ann", "bob"]
    names.push("cy")
    names.insert(0, "zed")
    names[1] = "al"
    println(names)
    more = ["ann", "bo", "cy"]
    short = [n for n in more if n.len() == 2]
    pair = [more[0], more[1]]
    view = more[1..]
    again = []
    for n in view:
        again.push(n)
    println(short, pair, again)
