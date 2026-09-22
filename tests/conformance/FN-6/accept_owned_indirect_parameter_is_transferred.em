#$ test: run-pass
#$ rules: FN-1, FN-6, FN-6a, OWN-2, OWN-3, TST-19
#$ profiles: debug, release, shipping
#$ stdout: 7

# An indirect callable keeps the declared ownership mode of each parameter.
# `resource` is moved into `consume`, so only the callee runs Resource.drop.
struct Resource:
    value: i32

    fn drop(mut self):
        println(self.value)

fn consume(owned resource: Resource):
    _value = resource.value

fn main():
    operation: fn(owned Resource) = consume
    resource = Resource(7)
    operation(resource)
