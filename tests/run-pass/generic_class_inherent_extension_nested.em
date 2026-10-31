#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, development, release
#$ stdout: 2

[Payload]
class Envelope:
    value: Payload

extend[Element] Envelope[Array[Element]]:
    fn count() -> int:
        return this.value.len()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    values.push(9)
    envelope = Envelope[Array[i32]](values)
    println(envelope.count())
