#include <cstdint>
#include <cstdio>
#include <vector>
struct Shape { virtual double area() const = 0; virtual ~Shape() {} };
struct Circle : Shape { double r; explicit Circle(double r) : r(r) {} double area() const override { return 3.0 * r * r; } };
struct Square : Shape { double s; explicit Square(double s) : s(s) {} double area() const override { return s * s; } };
int main() {
    std::vector<Shape*> shapes;
    for (int64_t i = 0; i < 1000000; i++) {
        if (i % 2 == 0) shapes.push_back(new Circle(1.0));
        else shapes.push_back(new Square(2.0));
    }
    double total = 0.0;
    for (int round = 0; round < 20; round++)
        for (Shape* s : shapes) total = total + s->area();
    std::printf("%lld\n", (long long)total);
    return 0;
}
