#$ test: run-pass
#$ rules: FN-1, OWN-2, OWN-3
# An `owned` argument transfers ownership to the callee. The parameter must
# therefore drop on the callee's exits unless it was moved onward. The two
# calls also pin an owned parameter's conditional drop flag: true clears it
# after moving into `_moved`; false leaves it set for the parameter itself.

struct Resource:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn finish(owned resource: Resource, move_again: bool):
    if move_again:
        _moved = resource

fn main():
    finish(Resource(1), true)
    finish(Resource(2), false)
#$ stdout: 1
#$ 2
