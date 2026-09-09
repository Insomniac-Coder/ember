#$ test: compile-fail
#$ rules: RNG-10, RNG-9
## "Any other route is `unsafe` and is `T.new_unchecked(v)`."

type Percent = u8 in 0 ..= 100

fn main():
    p = Percent.new_unchecked(200)    #$ error[E3100]: `Percent.new_unchecked` requires `unsafe`
    print(1)
