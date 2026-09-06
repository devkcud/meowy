#include "meowy/context.hpp"

#include <asm/prctl.h>
#include <cerrno>
#include <cstdlib>
#include <sys/syscall.h>
#include <unistd.h>

#if !defined(__linux__) || !defined(__x86_64__) || defined(__ILP32__)
#error "The pinned context prototype supports Linux x86-64 LP64 only"
#endif

#if defined(__CET__) && (__CET__ & 2)
#error "The pinned context prototype does not support CET shadow stacks"
#endif

#if __has_feature(address_sanitizer)
#include <sanitizer/common_interface_defs.h>
#endif

namespace meowy::prototype::v0 {

struct ContextTransfer final {
public:
    void *context;
    void *data;
};

static_assert(sizeof(ContextTransfer) == 16);

extern "C" ContextTransfer meowy_jump_context_v0(void *, void *) noexcept;
extern "C" void *meowy_make_context_v0(void *, std::size_t, void (*)(ContextTransfer)) noexcept;

thread_local Context *Context::current = nullptr;

ContextResult Context::initialize(std::size_t bytes, Body entry, void *argument) noexcept {
    if (phase != ContextState::empty || memory.owns() || entry == nullptr || bytes < minimum_bytes) {
        return {ContextStatus::invalid, {}};
    }
    if (!supported()) {
        return {ContextStatus::unsupported, {}};
    }
    const auto result = memory.allocate(bytes);
    if (result.status != MemoryStatus::ok) {
        return {ContextStatus::memory_failed, result};
    }
    body = entry;
    data = argument;
    fiber = meowy_make_context_v0(memory.data() + memory.size().usable, memory.size().usable, enter);
    phase = ContextState::ready;
    return {};
}

[[clang::no_sanitize("address")]] ContextStatus Context::resume() noexcept {
    if (current != nullptr || (phase != ContextState::ready && phase != ContextState::suspended)) {
        return ContextStatus::invalid;
    }
    if (!same_thread()) {
        return ContextStatus::wrong_thread;
    }
    if (!supported()) {
        return ContextStatus::unsupported;
    }
    worker = pthread_self();
    pinned = true;
    phase = ContextState::running;
    current = this;
#if __has_feature(address_sanitizer)
    __sanitizer_start_switch_fiber(&caller_fake, memory.data(), memory.size().usable);
#endif
    const auto transfer = meowy_jump_context_v0(fiber, this);
#if __has_feature(address_sanitizer)
    __sanitizer_finish_switch_fiber(caller_fake, nullptr, nullptr);
#endif
    fiber = transfer.context;
    current = nullptr;
    return ContextStatus::ok;
}

[[clang::no_sanitize("address")]] ContextStatus Context::yield() noexcept {
    if (!same_thread()) {
        return ContextStatus::wrong_thread;
    }
    if (current != this || phase != ContextState::running) {
        return ContextStatus::invalid;
    }
    if (!supported()) {
        return ContextStatus::unsupported;
    }
    phase = ContextState::suspended;
    current = nullptr;
#if __has_feature(address_sanitizer)
    __sanitizer_start_switch_fiber(&fake, caller_bottom, caller_size);
#endif
    const auto transfer = meowy_jump_context_v0(caller, this);
#if __has_feature(address_sanitizer)
    __sanitizer_finish_switch_fiber(fake, &caller_bottom, &caller_size);
#endif
    caller = transfer.context;
    return ContextStatus::ok;
}

ContextResult Context::release() noexcept {
    if (phase == ContextState::running || phase == ContextState::suspended) {
        return {ContextStatus::invalid, {}};
    }
    if (!same_thread()) {
        return {ContextStatus::wrong_thread, {}};
    }
    const auto result = memory.release();
    if (result.status != MemoryStatus::ok) {
        return {ContextStatus::memory_failed, result};
    }
    phase = ContextState::empty;
    body = nullptr;
    data = nullptr;
    fiber = nullptr;
    caller = nullptr;
    fake = nullptr;
    caller_fake = nullptr;
    caller_bottom = nullptr;
    caller_size = 0;
    pinned = false;
    return {};
}

ContextState Context::state() const noexcept {
    return phase;
}

StackSize Context::size() const noexcept {
    return memory.size();
}

const std::byte *Context::bottom() const noexcept {
    return memory.data();
}

[[noreturn, clang::no_sanitize("address")]] void Context::enter(ContextTransfer transfer) noexcept {
    auto &self = *static_cast<Context *>(transfer.data);
    self.caller = transfer.context;
#if __has_feature(address_sanitizer)
    __sanitizer_finish_switch_fiber(nullptr, &self.caller_bottom, &self.caller_size);
#endif
    self.body(self, self.data);
    self.phase = ContextState::completed;
    current = nullptr;
    if (!supported()) {
        std::abort();
    }
#if __has_feature(address_sanitizer)
    __sanitizer_start_switch_fiber(nullptr, self.caller_bottom, self.caller_size);
#endif
    static_cast<void>(meowy_jump_context_v0(self.caller, &self));
    std::abort();
}

bool Context::supported() noexcept {
    unsigned long features = 0;
    if (syscall(SYS_arch_prctl, ARCH_SHSTK_STATUS, &features) == 0) {
        return (features & ARCH_SHSTK_SHSTK) == 0;
    }
    return errno == EINVAL || errno == ENOSYS || errno == ENOTSUP;
}

bool Context::same_thread() const noexcept {
    return !pinned || pthread_equal(worker, pthread_self()) != 0;
}

}
