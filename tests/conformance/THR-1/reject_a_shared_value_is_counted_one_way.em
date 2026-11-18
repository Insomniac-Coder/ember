#$ test: compile-fail
#$ rules: THR-1, RC-1
# `[THR-1]`, `[RC-1]` (`E7017`) — `thread_shared` of a value made otherwise, and `shares` of one made
# `thread_shared`: its owners are counted one way from its making.

class Names:
    n: int = 0

fn main():
    names = Names()
    _a = thread_shared names    #$ error[E7017]: was not made `thread_shared`
    shared = thread_shared Names()
    _b = shares shared    #$ error[E7017]: is `thread_shared`
