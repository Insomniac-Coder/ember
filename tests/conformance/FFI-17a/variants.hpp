// The C++ side of accept_a_variant_crosses_as_an_enum.em: `std::variant`s a header names with an
// alias, each alternative a value Ember has (ODR-198).
#pragma once
#include <variant>

struct Circle { double radius; };
struct Square { double side; };
using Shape = std::variant<Circle, Square>;

inline double area(const Shape& shape) noexcept {
    if (const Circle* circle = std::get_if<Circle>(&shape)) return 3.0 * circle->radius * circle->radius;
    return std::get<Square>(shape).side * std::get<Square>(shape).side;
}
inline Shape bigger(Shape shape) noexcept {
    if (const Circle* circle = std::get_if<Circle>(&shape)) return Square{circle->radius * 2};
    return Circle{std::get<Square>(shape).side};
}

using Number = std::variant<int, double>;
inline Number half(int n) noexcept {
    if (n % 2 == 0) return n / 2;
    return n / 2.0;
}
inline int kind_of(Number number) noexcept { return static_cast<int>(number.index()); }
