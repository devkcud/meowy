#pragma once

#include "meowy/stack_memory.hpp"

#include <pthread.h>

namespace meowy::prototype::v0 {

enum class ContextStatus : unsigned char { ok, invalid, wrong_thread, unsupported, memory_failed };
enum class ContextState : unsigned char { empty, ready, running, suspended, completed };

struct ContextResult final {
public:
    ContextStatus status = ContextStatus::ok;
    MemoryResult memory;
};

struct ContextTransfer;

class Context final {
public:
    using Body = void (*)(Context &, void *) noexcept;
    static constexpr std::size_t minimum_bytes = 65536;

    Context() = default;
    Context(const Context &) = delete;
    Context &operator=(const Context &) = delete;
    Context(Context &&) = delete;
    Context &operator=(Context &&) = delete;

    [[nodiscard]] ContextResult initialize(std::size_t bytes, Body body, void *data) noexcept;
    [[nodiscard]] ContextStatus resume() noexcept;
    [[nodiscard]] ContextStatus yield() noexcept;
    [[nodiscard]] ContextResult release() noexcept;
    [[nodiscard]] ContextState state() const noexcept;
    [[nodiscard]] StackSize size() const noexcept;
    [[nodiscard]] const std::byte *bottom() const noexcept;

private:
    [[noreturn]] static void enter(ContextTransfer transfer) noexcept;
    [[nodiscard]] static bool supported() noexcept;
    [[nodiscard]] bool same_thread() const noexcept;

    static thread_local Context *current;
    StackMemory memory;
    ContextState phase = ContextState::empty;
    Body body = nullptr;
    void *data = nullptr;
    void *fiber = nullptr;
    void *caller = nullptr;
    void *fake = nullptr;
    void *caller_fake = nullptr;
    const void *caller_bottom = nullptr;
    std::size_t caller_size = 0;
    pthread_t worker{};
    bool pinned = false;
};

}
