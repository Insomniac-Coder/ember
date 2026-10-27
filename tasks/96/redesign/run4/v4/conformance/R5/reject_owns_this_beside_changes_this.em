#$ test: compile-fail
#$ rules: R5
#$ help: fn finish() owns this -> u64:
## R5: `owns this` and `changes this` do not stand together: taking the object over already lets the
## method do anything with it (E2107).

class Job:
    id: u64

    fn finish() owns this changes this -> u64:    #$ error[E2107]: `finish` says `owns this`, which takes the whole object over and so may also change it; write `fn finish() owns this -> u64:`
        return this.id

fn main():
    j = Job(id=7)
    println(j.finish())
