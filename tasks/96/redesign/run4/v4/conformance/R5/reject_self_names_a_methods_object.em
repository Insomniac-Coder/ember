#$ test: compile-fail
#$ rules: R5
#$ help: fn reset() changes this:
## R5: inside a method its object is `this`, and a method does not list it among its parameters;
## `self` is not a word of Ember (owner, 2026-10-25; rules section 3, E1010).

class Counter:
    count: int = 0

    fn reset(self):    #$ error[E1010]: `self` is not a word of Ember: inside a method its object is `this`; write `fn reset() changes this:` for a method that changes it, and `this.count = 0` in its body
        self.count = 0

fn main():
    c = Counter()
    c.reset()
    println(c.count)
