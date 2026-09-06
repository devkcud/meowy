#pragma once

#include "meowy/cleanup.hpp"
#include "meowy/context.hpp"

#include <span>

namespace meowy::prototype::v0 {

enum class ScheduleStatus : unsigned char { ok, full, invalid, wrong_thread, context_failed };
enum class TaskState : unsigned char { vacant, runnable, running, waiting, settled };
enum class OutcomeKind : unsigned char { pending, completed, panicked, spawn_failed };

class Scheduler;
class Task;

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
    bool operator==(const TaskTicket &) const noexcept = default;
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
    TaskTicket parent;
    TaskTicket waiting;
    std::size_t children = 0;
};

struct PumpResult final {
public:
    ScheduleStatus status = ScheduleStatus::invalid;
    std::size_t resumed = 0;
    std::size_t runnable = 0;
    ContextStatus context = ContextStatus::ok;
    std::size_t waiting = 0;
};

struct Joined final {
public:
    ScheduleStatus status = ScheduleStatus::invalid;
    TaskOutcome outcome;
    ContextResult context;
};

class Task final {
public:
    Task(const Task &) = delete;
    Task &operator=(const Task &) = delete;
    Task(Task &&) = delete;
    Task &operator=(Task &&) = delete;
    [[nodiscard]] ContextStatus yield() noexcept;
    [[nodiscard]] Submission spawn(TaskBody body, void *data, TaskBody cleanup = nullptr) noexcept;
    [[nodiscard]] Joined join(TaskTicket child) noexcept;

private:
    friend class Scheduler;
    Task(Scheduler &scheduler, Context &context, TaskTicket ticket) noexcept;
    Scheduler &scheduler;
    Context &context;
    const TaskTicket ticket;
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
    Scheduler *owner = nullptr;
    std::size_t index = 0;
    TaskTicket parent;
    TaskTicket waiting;
    std::size_t children = 0;
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
    friend class Task;
    struct Activation;
    static void run(Context &context, void *data) noexcept;
    static Panic clean(void *data) noexcept;
    [[noreturn]] static void unjoined(std::string_view phase) noexcept;
    [[nodiscard]] ScheduleStatus access() const noexcept;
    [[nodiscard]] ScheduleStatus task_access(TaskTicket ticket) const noexcept;
    [[nodiscard]] Submission admit(TaskBody body, void *data, TaskBody cleanup, TaskTicket parent) noexcept;
    [[nodiscard]] Joined join_child(TaskTicket parent, TaskTicket child) noexcept;
    [[nodiscard]] Joined consume(TaskTicket ticket) noexcept;
    void wake(TaskTicket child) noexcept;
    [[nodiscard]] bool valid(TaskTicket ticket) const noexcept;
    [[nodiscard]] std::size_t runnable_count() const noexcept;
    [[nodiscard]] std::size_t waiting_count() const noexcept;

    const std::span<TaskSlot> slots;
    const std::size_t stack_bytes;
    std::size_t cursor = 0;
    std::uint64_t next = 1;
    const pthread_t worker;
    bool pumping = false;
    TaskTicket active;
    const bool configured;
};

}
