#$ test: compile-fail
#$ rules: SPN-5, BRW-1, LT-1, TST-25

[T]
fn first_mut(ref mut span: MutSpan[T]) -> Option[ref mut T]:
    iterator = span.iter_mut()
    item = iterator.next()
    _again = ref mut span #$ error[E3022]: `span` is already mutably borrowed
    return item

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    _item = first_mut[i32](ref mut values.as_mut_span())
