#$ test: run-pass
#$ rules: RNG-3, COST-3
## "Construction from a constant in range […] emits no check." A range type is
## `[COST-3]`'s *not observable*: erased to the representation, with the check
## living at the construction site — and here there is no check to live.

type Percent = u8 in 0 ..= 100

fn main():
    p: Percent = 100
    v: u32 = p
    print(v)
#$ stdout: 100
#$ assert-c: !contains("ember_panic")
