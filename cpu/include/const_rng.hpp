#pragma once

#include <array>

namespace evo {

class const_rng {
private:
    std::uint64_t state;

    static constexpr auto time_from_string(const char* str, int offset) {
        return static_cast<std::uint32_t>(str[offset] - '0') * 10 +
            static_cast<std::uint32_t>(str[offset + 1] - '0');
    }
public:
    static constexpr std::uint64_t M = std::uint64_t(1) << 48;
    static constexpr std::uint64_t A = 0x5DEECE66D;
    static constexpr std::uint64_t C = 11;

    constexpr const_rng(std::uint64_t s = get_seed()): state(s) {}

    constexpr std::uint64_t next(std::uint8_t bits) noexcept {
        state = ((A * state + C) % M);
        return state >> (48 - bits);
    }

    constexpr double next_double() noexcept {
        auto v = next(48);
        return static_cast<double>(v) / M;
    }

    template <typename T>
    constexpr T next(T min, T max) {
        return static_cast<T>(next_double() * (max - min) + min);
    }

    template <typename T, std::size_t sz>
    constexpr std::array<T, sz> next_values(T min, T max) {
        std::array<T, sz> dst {};
        for(auto& el : dst) {
            el = static_cast<T>(next_double() * (max - min) + min);
        }
        return dst;
    }

    static constexpr std::uint64_t get_seed() {
        auto t = __TIME__;
        return time_from_string(t, 0) * 60 * 60 + time_from_string(t, 3) * 60 + time_from_string(t, 6);
    }
};

}
