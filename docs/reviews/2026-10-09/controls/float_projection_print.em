from std.math import Float

interface Holder:
    type Item: Float + Display

fn show[H: Holder](x: H.Item):
    println(x)

fn main():
    pass
