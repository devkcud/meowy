#pragma once

#include "meowy/cleanup.hpp"
#include "meowy/context.hpp"

#include <span>

namespace meowy::prototype::v0 {

enum class ScheduleStatus : unsigned char { ok, full, invalid, wrong_thread, context_failed };
enum class TaskState : unsigned char { vacant, runnable, running, settled };
enum class OutcomeKind : unsigned char { pending, completed, panicked, spawn_failed };

class Scheduler;

class Task final {
public:
    Task(const Task &) = delete;
    Task &operator=(const Task &) = delete;
    Task(Task &&) = delete;
    Task &operator=(Task &&) = delete;
    [[nodiscard]] ContextStatus yield() noexcept;

private:
    friend class Scheduler;
    explicit Task(Context &context) noexcept;
    Context &context;
};

using TaskBody = Panic (*)(Task &, void *) noexcept;

struct TaskOutcome final {
public:
    OutcomeKind kind = OutcomeKind::pending;
    Panic panic;
    ContextResult context;
};

struct TaskTicket final {
public:
    const Scheduler *scheduler = nullptr;
    std::size_t index = 0;
    std::uint64_t id = 0;
};

struct Submission final {
public:
    ScheduleStatus status = ScheduleStatus::invalid;
    TaskTicket ticket;
    ContextResult context;
};

struct TaskInfo final {
public:
    ScheduleStatus status = ScheduleStatus::invalid;
    TaskState state = TaskState::vacant;
    TaskOutcome outcome;
};

struct PumpResult final {
public:
    ScheduleStatus status = ScheduleStatus::invalid;
    std::size_t resumed = 0;
    std::size_t runnable = 0;
    ContextStatus context = ContextStatus::ok;
};

struct Joined final {
public:
    ScheduleStatus status = ScheduleStatus::invalid;
    TaskOutcome outcome;
    ContextResult context;
};

class TaskSlot final {
public:
    TaskSlot() = default;
    [[nodiscard]] bool runnable() const noexcept;
    [[nodiscard]] std::uint64_t sequence() const noexcept;

private:
    friend class Scheduler;
    Context context;
    TaskState state = TaskState::vacant;
    TaskOutcome outcome;
    std::uint64_t id = 0;
    TaskBody body = nullptr;
    TaskBody cleanup = nullptr;
    void *data = nullptr;
};

[[nodiscard]] std::size_t select_task(std::span<const TaskSlot> slots, std::size_t cursor) noexcept;

class Scheduler final {
public:
    Scheduler(std::span<TaskSlot> slots, std::size_t stack_bytes) noexcept;
    Scheduler(const Scheduler &) = delete;
    Scheduler &operator=(const Scheduler &) = delete;
    Scheduler(Scheduler &&) = delete;
    Scheduler &operator=(Scheduler &&) = delete;

    [[nodiscard]] Submission submit(TaskBody body, void *data, TaskBody cleanup = nullptr) noexcept;
    [[nodiscard]] PumpResult pump(std::size_t limit) noexcept;
    [[nodiscard]] TaskInfo inspect(TaskTicket ticket) const noexcept;
    [[nodiscard]] Joined join(TaskTicket ticket) noexcept;

private:
    struct Activation;
    static void run(Context &context, void *data) noexcept;
    static Panic clean(void *data) noexcept;
    [[nodiscard]] ScheduleStatus access() const noexcept;
    [[nodiscard]] bool valid(TaskTicket ticket) const noexcept;
    [[nodiscard]] std::size_t runnable_count() const noexcept;

    const std::span<TaskSlot> slots;
    const std::size_t stack_bytes;
    std::size_t cursor = 0;
    std::uint64_t next = 1;
    const pthread_t worker;
    bool pumping = false;
    const bool configured;
};

}
