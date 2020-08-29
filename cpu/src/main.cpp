#include <algorithm>
#include <chrono>
#include <iostream>
#include <vector>
#include "const_rng.hpp"

#include <cmath> //sin

constexpr std::uint64_t count = 10;
constexpr std::uint64_t steps = 15000;

constexpr double min_x = 0;
constexpr double max_x = 1000;

constexpr double min_mut = 0.05;
constexpr double max_mut = 0.10;

/* can't be constexpr because sin/cos aren't, but that's fixable :^) */
double score(double x) {
    return (2*cos(0.039*x) + 5*sin(0.05*x) + 0.5*cos(0.01*x) + 10*sin(0.07*x) + 5*sin(0.1*x) + 5*sin(0.035*x))*10+500;
}

class Thing {
private:
    std::uint64_t _id;
    double _x;
public:
    constexpr Thing() {}
    constexpr Thing(std::uint64_t id, double x): _id(id), _x(x) {}

    constexpr std::uint64_t id() const noexcept { return _id; }

    constexpr double x() const noexcept { return _x; }
    double score() const noexcept { return ::score(_x); }

    constexpr void merge(evo::const_rng& rng, const Thing& other) noexcept {
        auto score_diff = score() - other.score();
        if(score_diff < 0) score_diff *= -1;
        _x = (_x + other.x()) / 2;
        mutate(rng, score_diff / 100);
    }

    constexpr void mutate(evo::const_rng& rng, double add = 0) noexcept {
        double range = (max_mut - min_mut + add);
        _x = _x + (rng.next(min_mut, min_mut + range) - (range / 2));
    }

    void print() const noexcept {
        std::cout << "(" << _id << ", x=" << _x << ", score=" << score() << ")";
    }
};

using Container = std::array<Thing, count>;

struct Stats { double min; double max; double sum; };

constexpr Stats measure(const Container& vec) {
    auto min = +100000.0;
    auto max = -100000.0;
    auto sum = 0.0;
    for(auto it = vec.begin(); it != vec.end(); it++) {
        auto score = (*it).score();
        if(score < min) min = score;
        if(score > max) max = score;
        sum += score;
    }
    return Stats { min, max, sum };
}

constexpr Container compute(std::uint64_t seed) {
    evo::const_rng rng(seed);

    Container things = {};

    for(auto i = 0; i < count; i++) {
        Thing t(i, rng.next(min_x, max_x));
        things[i] = t;
    }

    for(auto i = 0; i < steps; i++) {
        std::sort(things.begin(), things.end(), [](auto a, auto b) {
            return a.score() < b.score();
        });
        auto& best = things.back();

        for(auto it = things.begin(); it != things.end() - 1; it++) {
            (*it).merge(rng, best);
        }
        //best.mutate(rng);
    }

    return things;
}

int main() {
//    uint64_t seed = std::chrono::duration_cast<std::chrono::microseconds>(
//            std::chrono::system_clock::now().time_since_epoch()
//    ).count();
    constexpr uint64_t seed = 123;
    
    std::cout << "Count: " << count << ", steps: " << steps << std::endl;
    std::cout << "Seed: " << seed << std::endl;

    /* constexpr */ const auto res = compute(seed);
    //   ^ my std::sort isn't constexpr ;-;
    //     no compile-time algorithm execution for me
    const auto s = measure(res);
    std::cout << "Results:" << std::endl;
    std::cout << "  min:  " << s.min << std::endl;
    std::cout << "  max:  " << s.max << std::endl;
    std::cout << "  avg:  " << (s.sum / count) << std::endl;
    std::cout << "  best: "; res.back().print(); std::cout << std::endl;
}
