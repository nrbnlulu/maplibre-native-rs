// Derived from MapLibre Native core-a33d3f00fb9dbf9117ce18edd799aa0ca5bcce5b.
// SPDX-License-Identifier: BSD-2-Clause
// Copyright notices and terms: ../android/LICENSE-MAPLIBRE
#pragma once



#include <mln/util/chrono.hpp>
#include <mln/util/run_loop.hpp>

#include <atomic>
#include <list>
#include <memory>
#include <mutex>
#include <unordered_map>

struct ALooper;

namespace mln {
namespace util {

using WatchCallback = std::function<void(int, RunLoop::Event)>;

template <typename T>
class Thread;
class Alarm;

class RunLoop::Impl {
public:
    class Runnable {
    public:
        virtual ~Runnable() = default;

        virtual void runTask() = 0;
        virtual TimePoint dueTime() const = 0;
    };

    Impl(RunLoop*, RunLoop::Type);
    ~Impl();

    void wake();

    void addRunnable(Runnable*);
    void removeRunnable(Runnable*);

    Milliseconds processRunnables();

    void waitForEmpty();

    ALooper* loop = nullptr;
    RunLoop* runLoop = nullptr;
    std::atomic<bool> running = false;
    std::atomic_flag coalesce = ATOMIC_FLAG_INIT;

    std::unordered_map<int, WatchCallback> readPoll;

private:
    friend RunLoop;

    int fds[2];

    std::unique_ptr<Thread<Alarm>> alarm;

    std::mutex mutex;
    std::condition_variable cvEmpty;
    std::list<Runnable*> runnables;
};

} // namespace util
} // namespace mln
