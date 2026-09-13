#$ test: run-pass
#$ rules: SPN-5, LT-1, TST-25

fn first[T](span: Span[T]) -> Option[ref T]:
    iterator = span.iter()
    return iterator.next()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    match first[i32](values.as_span()):
        Some(item):
            println(item)
        None:
            pass
#$ stdout: 7
