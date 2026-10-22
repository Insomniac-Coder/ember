#$ test: run-pass
#$ rules: MNG-1
#$ stdout: 3 4
#$ assert-c: contains("struct em_8Bag_iter {")
#$ assert-c: contains("int64_t em_3Bag4iter(")
# `[MNG-1]` — a struct `Bag_iter` and the method `Bag.iter` of one module are `em_8Bag_iter` and
# `em_3Bag4iter`: the length prefixes keep `_` inside a name apart from the step between two
# components. With `_` between components both were `em_Bag_iter`, one C name for a type and a
# function, which the C compiler refused (D-782). The method is called from two places, so it is
# not inlined into `main`.

struct Bag_iter:
    n: int

struct Bag:
    n: int

extend Bag:
    fn iter(self) -> int:
        return self.n + 1

fn main():
    b = Bag(n = 2)
    println(b.iter(), Bag(n = 3).iter() + Bag_iter(n = 0).n)
