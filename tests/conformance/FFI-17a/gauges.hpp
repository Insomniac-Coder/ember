// The C++ side of `accept_an_empty_shared_pointer_has_a_non_panicking_get.em`: a
// `std::shared_ptr` that shares an object, and one that is empty.
#pragma once
#include <memory>

class Gauge {
public:
    explicit Gauge(int reading) noexcept : reading_(reading) {}
    int reading() const noexcept { return reading_; }

private:
    int reading_;
};

inline std::shared_ptr<Gauge> gauge(int reading) { return std::make_shared<Gauge>(reading); }
inline std::shared_ptr<Gauge> no_gauge() noexcept { return nullptr; }
