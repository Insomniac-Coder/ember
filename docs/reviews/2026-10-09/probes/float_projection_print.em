from std.math import Float

interface Holder:
    type Item: Float

fn show[H: Holder](x: H.Item):
    println(x)

fn main():
    pass
