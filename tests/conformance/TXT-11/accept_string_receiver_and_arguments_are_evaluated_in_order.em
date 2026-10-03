#$ test: run-pass
#$ rules: TXT-11, EXP-1, BCK-5
#$ profiles: debug, release, shipping
#$ stdout: receiver
#$ stdout: offset
#$ stdout: character
#$ stdout: éa
fn receiver_index() -> int:
    println("receiver")
    return 0

fn offset() -> int:
    println("offset")
    return 0

fn character() -> char:
    println("character")
    return 'é'

fn main():
    texts: Array[String] = ["a"]
    texts[receiver_index()].insert(offset(), character())
    println(texts[0])
