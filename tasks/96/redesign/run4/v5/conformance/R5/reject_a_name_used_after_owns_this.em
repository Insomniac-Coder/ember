#$ test: compile-fail
#$ rules: OWN-3
#$ help: id = j.id
## design rules: R5, R1
## R5: `owns this` takes the object over, and the caller's name ends at the call (E3040).

class Job:
    id: u64

    fn finish() owns this -> u64:
        return this.id

fn main():
    j = Job(id=7)
    n = j.finish()
    println(j.id, n)    #$ error[E3040]: `j` was handed over to `finish` on line 14 (its first line says `owns this`) and `j.id` reads it on line 15; read it before the call (`id = j.id` above line 14, and `println(id, n)` as line 15)
