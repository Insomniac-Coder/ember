#$ test: run-pass
#$ rules: HEAP-1, DRP-6, DRP-2
# Nested unique owners recursively destroy the payload before freeing each
# allocation: Resource, inner Box allocation, then outer Box allocation. The
# inner Box drops in its own glue function (D-358: a Box payload that owns
# another opens a level of nesting), called before the outer Box is freed.

struct Resource:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn main():
    _outer = Box(Box(Resource(5)))
    println(0)
#$ stdout: 0
#$ 5
#$ assert-c-order: "em_Resource_drop(&" then "ember_free((*value)"
#$ assert-c-order: "em_drop_glue_0(&" then "ember_free(_1,"
#$ assert-c-count: contains("ember_free(") == 2
