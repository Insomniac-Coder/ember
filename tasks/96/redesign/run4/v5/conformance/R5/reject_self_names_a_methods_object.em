#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `fn reset():` in place of line 10, and `this.count = 0` in place of line 11
## design rules: R5
## R5: inside a method its object is `this`, and a method does not list it among its parameters;
## `self` is not a word of Ember (owner, 2026-10-25; rules section 3, E1010).

class Counter:
    count: int = 0

    fn reset(self):    #$ error[E1010]: `self` is not a word of Ember: inside a method its object is `this`
        self.count = 0

fn main():
    c = Counter()
    c.reset()
    println(c.count)
