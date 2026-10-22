// The C++ side of accept_operators_of_a_template_instantiation.em: a class template whose
// operators an instantiation's import gives.
#pragma once

template <typename T> struct Pair {
    T first;
    T second;
    Pair operator+(const Pair& other) const noexcept { return Pair{first + other.first, second + other.second}; }
    bool operator==(const Pair& other) const noexcept { return first == other.first && second == other.second; }
    T sum() const noexcept { return first + second; }
};
