#$ test: run-pass
#$ rules: FN-5, EXP-1, FN-1
#$ profiles: debug, release, shipping
#$ stdout: 23 2
#$ 12
#$ true 3
# Written arguments run once in source order, then missing defaults run in
# parameter order. A borrowed record remains the caller's record, and a mut
# parameter still writes back through the caller's original place.

struct Counter:
    total: int

extend Counter:
    fn next(mut self) -> int:
        self.total += 1
        return self.total

struct Payload:
    a: int
    b: int
    c: int

fn compose(a: int, b: int = a * 10, c: int = b + a) -> int:
    return a + b + c

fn double(mut value: Payload, amount: int = value.b):
    value.b += amount

fn same_field(value: Payload, selected: ref int = ref value.b) -> bool:
    return selected is ref value.b

fn main():
    counter = Counter(total=0)
    println(compose(c=counter.next(), a=counter.next()), counter.total)
    item = Payload(a=2, b=6, c=4)
    double(item)
    println(item.b)
    println(same_field(item), item.a + item.c - 3)
