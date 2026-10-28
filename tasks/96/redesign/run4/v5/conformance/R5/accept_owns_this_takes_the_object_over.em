#$ test: run-pass
#$ rules: FN-4
#$ stdout: 7
## design rules: R5
## R5: `owns this` takes the object over; a caller that does not use the name again is accepted.

class Job:
    id: u64

    fn finish() owns this -> u64:
        return this.id

fn main():
    j = Job(id=7)
    println(j.finish())
