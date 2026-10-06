enum Shape:
    Circle(float)
    Rect(float, float)
    Tri(float, float)

fn area(s: Shape) -> float:
    match s:
        Circle(r):
            return 3.0 * r * r
        Rect(w, h):
            return w * h
        Tri(b, h):
            return 0.5 * b * h

fn main():
    shapes: Array[Shape] = []
    for i in 0..1000000:
        k = i % 3
        if k == 0:
            shapes.push(Shape.Circle(1.0))
        elif k == 1:
            shapes.push(Shape.Rect(2.0, 3.0))
        else:
            shapes.push(Shape.Tri(4.0, 5.0))
    total = 0.0
    for round in 0..100:
        for s in shapes:
            total = total + area(s)
    println(total as int)
