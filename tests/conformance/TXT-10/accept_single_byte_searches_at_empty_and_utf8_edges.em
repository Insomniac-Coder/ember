#$ test: run-pass
#$ rules: TXT-10, TXT-11, TXT-4
#$ profiles: debug, release, shipping
#$ stdout: None None Some(0) None Some(2) Some(5)
#$ 2 0 é!é! !x!
#$ |é|||
# A single-byte search handles an empty backing buffer, the first/last byte,
# UTF-8 byte offsets and NUL. Repeated searches stop at the end of the view.

fn main():
    empty: String = String.with_capacity(0)
    println(empty.find("x"), "".find("x"), "x".find("x"), "x".find("y"), "é,x,\0".find(","), "é,x,\0".find("\0"))
    println("é,é,".count(","), empty.count(","), "é,é,".replace(",", "!"), "x".replace("", "!"))
    for part in ",é,,".split(","):
        print(part)
        print("|")
    println()
