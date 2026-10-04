#$ test: run-pass
#$ rules: TXT-11, EXP-1, BCK-5, HEAP-8
#$ profiles: debug, release, shipping
#$ assert-c: contains("vec_extend_inline(")
#$ stdout: receiver
#$ stdout: piece
#$ stdout: 10
#$ stdout: é🙂abc

fn receiver_index() -> int:
    println("receiver")
    return 0

fn piece() -> str:
    println("piece")
    return "é\0🙂"

fn main():
    texts: Array[String] = [String.with_capacity(8)]
    capacity = texts[0].capacity()
    texts[receiver_index()].push_str(piece())
    assert(texts[0].len() == 7)
    assert(texts[0].capacity() == capacity)
    texts[0].push_str("")
    assert(texts[0].len() == 7)
    assert(texts[0].capacity() == capacity)
    texts[0].push_str("abc")
    assert(texts[0].capacity() >= 10)
    println(texts[0].len())
    assert(texts[0].remove(2) == '\0')
    println(texts[0])
