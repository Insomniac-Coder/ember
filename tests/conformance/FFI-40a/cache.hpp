// The C++ side of the FFI-40a tests: a class whose const methods change its mutable state.
#pragma once

class Cache {
public:
    Cache() noexcept {}
    ~Cache() {}
    int lookup(int key) const noexcept {
        ++lookups_;
        return key * 2;
    }
    int lookups() const noexcept { return lookups_; }

private:
    mutable int lookups_ = 0;
};
