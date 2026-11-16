// Derived from MapLibre Native core-a33d3f00fb9dbf9117ce18edd799aa0ca5bcce5b.
// SPDX-License-Identifier: BSD-2-Clause
// Copyright notices and terms: ../android/LICENSE-MAPLIBRE
#if defined(__ANDROID__)
#include <mln/util/logging.hpp>
#include <mln/util/platform.hpp>
#include <mln/platform/thread.hpp>

#include <sys/prctl.h>
#include <sys/resource.h>

#include <cassert>

// Implementation based on Chromium's platform_thread_android.cc.

namespace mln {
namespace platform {



std::string getCurrentThreadName() {
    char name[32] = "unknown";

    if (prctl(PR_GET_NAME, name) == -1) {
        Log::Warning(Event::General, "Couldn't get thread name");
    }

    return name;
}

void setCurrentThreadName(const std::string& name) {
    if (prctl(PR_SET_NAME, name.c_str()) == -1) {
        Log::Warning(Event::General, "Couldn't set thread name");
    }
}

void makeThreadLowPriority() {
    // ANDROID_PRIORITY_LOWEST = 19
    //
    // Supposedly would set the priority for the whole process, but
    // on Linux/Android it only sets for the current thread.
    setpriority(PRIO_PROCESS, 0, 19);
}

void setCurrentThreadPriority(double priority) {
    if (priority < -20 || priority > 19) {
        Log::Warning(Event::General, "Couldn't set thread priority");
        return;
    }
    setpriority(PRIO_PROCESS, 0, int(priority));
}

// This standalone platform uses native networking, image decoding and text
// conversion, so its worker threads never attach to MapLibre's Java module.
void attachThread() {}

void detachThread() {}

} // namespace platform
} // namespace mln

#endif
