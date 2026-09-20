#$ test: run-pass
#$ rules: TYP-16, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 2

class Envelope[Payload]:
    value: Payload

extend[Element] Envelope[Array[Element]]:
    fn count(self) -> usize:
        return self.value.len()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    values.push(9)
    envelope = Envelope[Array[i32]](values)
    println(envelope.count())
