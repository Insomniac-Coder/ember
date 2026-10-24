# A8, proposed: a graph whose nodes are owned by one list; roads are weak, and `into` is kept in step
# with `roads`, so a city that dies is taken out of every list that names it.
class City:
    name: String
    roads: Array[weak City] = []
    into: Array[weak City] = backlink(roads)

fn main():
    cities = [City("a"), City("b")]
    cities[0].roads.push(cities[1])
    cities[1].roads.push(cities[0])
    println(cities[1].into.len())
