#$ test: compile-fail
#$ rules: R5
#$ help: fn finish() owns this -> u64:
## R5, rules section 3's removed words: `owned self` is gone; a method that takes its object over says
## `owns this` after the brackets (E1010; completeness G8).

class Job:
    id: u64

    fn finish(owned self) -> u64:    #$ error[E1010]: `owned self` is not Ember: a method that takes its object over says so after the brackets; write `fn finish() owns this -> u64:`, and `return this.id` in its body
        return self.id

fn main():
    j = Job(id=7)
    println(j.finish())
