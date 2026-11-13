// The C++ side of the FFI-36 tests of a class with no public destructor: `destroy_enemy` ends one.
#pragma once

inline int destroyed = 0;

class Enemy {
public:
    explicit Enemy(int hp) noexcept : hp_(hp) {}
    int hp() const noexcept { return hp_; }
    friend void destroy_enemy(Enemy* enemy) noexcept;

private:
    ~Enemy() {}
    int hp_;
};

inline void destroy_enemy(Enemy* enemy) noexcept { ++destroyed; delete enemy; }
inline Enemy* spawn(int hp) noexcept { return new Enemy(hp); }
inline int destroyed_count() noexcept { return destroyed; }
