#$ test: compile-fail
#$ rules: DRP-7
# The counted-loop item is iteration-local; break ends it while the outer
# Holder's destructor still reads through the borrowed reference.

struct CounterHolder:
    borrowed: ref i32
    marker: i32

    fn drop(mut self):
        println(self.borrowed)

fn main():
    _outer: Option[CounterHolder] = None
    first: i32 = 0
    last: i32 = 1
    for i in first..last:
        _outer = Some(CounterHolder(ref i, 0)) #$ error[E3060]:
        break
