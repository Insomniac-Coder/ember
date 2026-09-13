#$ test: compile-fail
#$ rules: SPN-5, BRW-1, LT-1, TST-25

fn first_mut[T](mut span: MutSpan[T]) -> Option[ref mut T]:
    iterator = span.iter_mut()
    item = iterator.next()
    _again = span.reborrow() #$ error[E3022]: `span` is already mutably borrowed
    return item

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    _item = first_mut[i32](values.as_mut_span())
