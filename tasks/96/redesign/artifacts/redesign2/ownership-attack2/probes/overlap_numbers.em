# overlap_numbers.em: one struct passed as two written parameters (the report's rule 5 accepts this
# when the callee writes only numbers). Does today's Ember accept it?
struct Counter:
    n: int

fn reset_then_set(mut a: Counter, mut b: Counter) -> int:
    a.n = 0
    b.n = 1
    return a.n

fn main():
    c = Counter(n=5)
    println(reset_then_set(c, c))
