// The C++ header of `[DIA-5]`'s class case: a class with no default constructor.
#pragma once

class Gauge {
public:
    explicit Gauge(int scale) noexcept : scale_(scale) {}
    virtual ~Gauge() {}
    virtual int read() const noexcept { return scale_; }

private:
    int scale_;
};
