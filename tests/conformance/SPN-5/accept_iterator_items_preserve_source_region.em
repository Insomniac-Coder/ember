#$ test: run-pass
#$ rules: SPN-5, LT-1, TST-25

fn first[T](span: Span[T]) -> ref T:
    iterator = span.iter()
    match iterator.next():
        Some(item):
            return item
        None:
            unsafe:
                return span.get_unchecked(0)

fn first_mut[T](mut span: MutSpan[T]) -> ref mut T:
    iterator = span.iter_mut()
    match iterator.next():
        Some(item):
            return item
        None:
            unsafe:
                return span.get_unchecked(0)

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    println(first[i32](values.as_span()))
    item = first_mut[i32](values.as_mut_span())
    replacement: i32 = 9
    item = ref mut replacement
    println(values[0])
#$ stdout: 7
#$ stdout: 9
