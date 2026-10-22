enum Event:
    Joined(String)
    Left(String)

fn main():
    count = 0
    e = Event.Joined("ada")
    match e:
        Joined(_) => count += 1
        Left(name) => println(name)
    println(count)
