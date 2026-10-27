#$ test: compile-fail
#$ rules: R5
#$ help: fn finish() owns this -> u64:
## R5: a first line says its parameters, then `owns this`, then the result type, then `from`, in that
## order, so every first line reads the same way (E0118, `V5/rules.md` section 3's message). With
## `changes` and `reads` gone (verdict 86), these are all the parts it has.

class Job:
    id: u64

    fn finish() -> u64 owns this:    #$ error[E0118]: the parts of `finish`'s first line are out of order; write `fn finish() owns this -> u64:`
        return this.id

fn main():
    j = Job(id=7)
    println(j.finish())
