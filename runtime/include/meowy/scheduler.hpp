#pragma once

#include "meowy/cleanup.hpp"
#include "meowy/context.hpp"
#include "meowy/owned.hpp"

#include <array>
#include <span>

namespace meowy::prototype::v0 {

enum class ScheduleStatus : unsigned char { ok, full, invalid, wrong_thread, context_failed, storage_failed };
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
    OwnedStatus storage = OwnedStatus::ok;
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
    OwnedStatus storage = OwnedStatus::ok;
};

struct TaskInfo final {
public:
    ScheduleStatus status = ScheduleStatus::invalid;
    TaskState state = TaskState::vacant;
    TaskOutcome outcome;
    TaskTicket parent;
    TaskTicket waiting;
    std::size_t children = 0;
    bool has_result = false;
    std::size_t scopes = 0;
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
    OwnedStatus storage = OwnedStatus::ok;
};

class ScopeMark final {
public:
    ScopeMark() = default;

private:
    friend class Scheduler;
    TaskTicket parent;
    std::uint64_t id = 0;
};

struct ScopeOpen final {
public:
    ScheduleStatus status = ScheduleStatus::invalid;
    ScopeMark mark;
};

struct ScopeClose final {
public:
    ScheduleStatus status = ScheduleStatus::invalid;
    std::size_t joined = 0;
    std::size_t panicked = 0;
    std::size_t spawn_failed = 0;
    TaskOutcome first_failure{};
    TaskTicket pending{};
    Joined pending_result{};
};

class Task final {
public:
    static constexpr std::size_t scope_limit = 16;
    Task(const Task &) = delete;
    Task &operator=(const Task &) = delete;
    Task(Task &&) = delete;
    Task &operator=(Task &&) = delete;
    [[nodiscard]] ContextStatus yield() noexcept;
    [[nodiscard]] Submission spawn(TaskBody body, void *data, TaskBody cleanup = nullptr) noexcept;
    [[nodiscard]] Submission spawn_owned(TaskBody body, Owned &capture) noexcept;
    [[nodiscard]] Joined join(TaskTicket child) noexcept;
    [[nodiscard]] Joined join_owned(TaskTicket child, Owned &destination) noexcept;
    [[nodiscard]] OwnedStatus set_result(Owned &value) noexcept;
    [[nodiscard]] OwnedStatus emit_capture() noexcept;
    [[nodiscard]] ScopeOpen mark() noexcept;
    [[nodiscard]] ScopeClose close(ScopeMark mark) noexcept;

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
    TaskSlot(std::span<std::byte> capture, std::span<std::byte> result) noexcept;
    [[nodiscard]] bool runnable() const noexcept;
    [[nodiscard]] std::uint64_t sequence() const noexcept;

private:
    friend class Scheduler;
    friend class Task;
    struct Scope final {
    public:
        std::uint64_t id = 0;
        std::uint64_t boundary = 0;
        std::size_t joined = 0;
        std::size_t panicked = 0;
        std::size_t spawn_failed = 0;
        TaskOutcome first_failure{};
    };
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
    Owned capture;
    Owned result;
    bool owned = false;
    bool producing = false;
    std::array<Scope, Task::scope_limit> scopes{};
    std::size_t scope_depth = 0;
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
    [[nodiscard]] Submission submit_owned(TaskBody body, Owned &capture) noexcept;
    [[nodiscard]] PumpResult pump(std::size_t limit) noexcept;
    [[nodiscard]] TaskInfo inspect(TaskTicket ticket) const noexcept;
    [[nodiscard]] Joined join(TaskTicket ticket) noexcept;
    [[nodiscard]] Joined join_owned(TaskTicket ticket, Owned &destination) noexcept;

private:
    friend class Task;
    struct Activation;
    static void run(Context &context, void *data) noexcept;
    static Panic clean(void *data) noexcept;
    static Panic drop_capture(void *data) noexcept;
    static Panic drop_result(void *data) noexcept;
    [[noreturn]] static void unfinished(std::string_view phase, std::string_view pending) noexcept;
    [[nodiscard]] ScheduleStatus access() const noexcept;
    [[nodiscard]] ScheduleStatus task_access(TaskTicket ticket) const noexcept;
    [[nodiscard]] Submission admit(TaskBody body, void *data, TaskBody cleanup, TaskTicket parent,
                                   Owned *capture = nullptr) noexcept;
    [[nodiscard]] Joined join_child(TaskTicket parent, TaskTicket child, Owned *destination = nullptr,
                                    bool discard = false) noexcept;
    [[nodiscard]] Joined consume(TaskTicket ticket, Owned *destination = nullptr, bool discard = false) noexcept;
    [[nodiscard]] ScopeOpen mark(TaskTicket parent) noexcept;
    [[nodiscard]] ScopeClose close(TaskTicket parent, ScopeMark mark) noexcept;
    void wake(TaskTicket child) noexcept;
    [[nodiscard]] bool valid(TaskTicket ticket) const noexcept;
    [[nodiscard]] std::size_t runnable_count() const noexcept;
    [[nodiscard]] std::size_t waiting_count() const noexcept;

    const std::span<TaskSlot> slots;
    const std::size_t stack_bytes;
    std::size_t cursor = 0;
    std::uint64_t next = 1;
    std::uint64_t next_scope = 1;
    const pthread_t worker;
    bool pumping = false;
    bool owning = false;
    TaskTicket active;
    const bool configured;
};

}
