// The C++ side of accept_a_raw_pointer_result_is_adopted.em.
#pragma once

class Widget {
public:
    explicit Widget(int size) noexcept : size_(size) {}
    ~Widget() { ++freed(); }
    int size() const noexcept { return size_; }
    static int& freed() noexcept { static int count = 0; return count; }

private:
    int size_;
};

inline Widget* make_widget(int size) noexcept { return new Widget(size); }
