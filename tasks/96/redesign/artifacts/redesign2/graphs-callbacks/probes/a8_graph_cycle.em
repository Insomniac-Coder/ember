# A8: a graph with cycles, written as Python does: cities that list their roads both ways.
class City:
    name: String
    roads: Array[City] = []

fn main():
    a = City("a")
    b = City("b")
    a.roads.push(b)
    b.roads.push(a)
    println(a.roads.len() + b.roads.len())
