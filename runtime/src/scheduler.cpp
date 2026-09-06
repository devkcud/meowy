#include "meowy/scheduler.hpp"

#include <array>
#include <cstdlib>
#include <limits>
#include <unistd.h>

namespace meowy::prototype::v0 {

struct Scheduler::Activation final {
public:
    Task task;
    TaskSlot &slot;
};

Task::Task(Context &value) noexcept : context(value) {}

ContextStatus Task::yield() noexcept {
    return context.yield();
}

bool TaskSlot::runnable() const noexcept {
    return state == TaskState::runnable;
}

std::uint64_t TaskSlot::sequence() const noexcept {
    return id;
}

Scheduler::Scheduler(std::span<TaskSlot> storage, std::size_t bytes) noexcept
    : slots(storage), stack_bytes(bytes), worker(pthread_self()), configured([storage, bytes] {
          const auto page = sysconf(_SC_PAGESIZE);
          return !storage.empty() && bytes >= Context::minimum_bytes && page > 0 &&
                 StackMemory::plan(bytes, static_cast<std::size_t>(page)).result.status == MemoryStatus::ok;
      }()) {}

Submission Scheduler::submit(TaskBody body, void *data, TaskBody cleanup) noexcept {
    const auto status = access();
    if (status != ScheduleStatus::ok || body == nullptr) {
        return {status == ScheduleStatus::ok ? ScheduleStatus::invalid : status, {}, {}};
    }
    if (next == std::numeric_limits<std::uint64_t>::max()) {
        return {ScheduleStatus::full, {}, {}};
    }
    for (std::size_t index = 0; index < slots.size(); ++index) {
        auto &slot = slots[index];
        if (slot.state != TaskState::vacant) {
            continue;
        }
        slot.id = next++;
        slot.body = body;
        slot.cleanup = cleanup;
        slot.data = data;
        slot.outcome = {};
        const TaskTicket ticket{this, index, slot.id};
        const auto result = slot.context.initialize(stack_bytes, run, &slot);
        if (result.status != ContextStatus::ok) {
            slot.state = TaskState::settled;
            slot.outcome = {OutcomeKind::spawn_failed, {}, result};
            return {ScheduleStatus::context_failed, ticket, result};
        }
        slot.state = TaskState::runnable;
        return {ScheduleStatus::ok, ticket, {}};
    }
    return {ScheduleStatus::full, {}, {}};
}

PumpResult Scheduler::pump(std::size_t limit) noexcept {
    const auto status = access();
    if (status != ScheduleStatus::ok) {
        return {status, 0, 0, ContextStatus::ok};
    }
    PumpResult result{ScheduleStatus::ok, 0, 0, ContextStatus::ok};
    pumping = true;
    while (result.resumed < limit) {
        const auto index = select_task(slots, cursor);
        if (index == slots.size()) {
            break;
        }
        if (index > slots.size() || !slots[index].runnable()) {
            result.status = ScheduleStatus::invalid;
            break;
        }
        auto &slot = slots[index];
        slot.state = TaskState::running;
        const auto resumed = slot.context.resume();
        if (resumed != ContextStatus::ok) {
            slot.state = TaskState::runnable;
            result.status = ScheduleStatus::context_failed;
            result.context = resumed;
            break;
        }
        ++result.resumed;
        cursor = index + 1 == slots.size() ? 0 : index + 1;
        if (slot.context.state() == ContextState::completed) {
            slot.state = TaskState::settled;
        } else if (slot.context.state() == ContextState::suspended) {
            slot.state = TaskState::runnable;
        } else {
            result.status = ScheduleStatus::invalid;
            break;
        }
    }
    pumping = false;
    result.runnable = runnable_count();
    return result;
}

TaskInfo Scheduler::inspect(TaskTicket ticket) const noexcept {
    const auto status = access();
    if (status != ScheduleStatus::ok || !valid(ticket)) {
        return {status == ScheduleStatus::ok ? ScheduleStatus::invalid : status, TaskState::vacant, {}};
    }
    const auto &slot = slots[ticket.index];
    return {ScheduleStatus::ok, slot.state, slot.outcome};
}

Joined Scheduler::join(TaskTicket ticket) noexcept {
    const auto status = access();
    if (status != ScheduleStatus::ok || !valid(ticket) || slots[ticket.index].state != TaskState::settled) {
        return {status == ScheduleStatus::ok ? ScheduleStatus::invalid : status, {}, {}};
    }
    auto &slot = slots[ticket.index];
    const auto result = slot.context.release();
    if (result.status != ContextStatus::ok) {
        return {ScheduleStatus::context_failed, slot.outcome, result};
    }
    const auto outcome = slot.outcome;
    slot.state = TaskState::vacant;
    slot.outcome = {};
    slot.body = nullptr;
    slot.cleanup = nullptr;
    slot.data = nullptr;
    return {ScheduleStatus::ok, outcome, {}};
}

void Scheduler::run(Context &context, void *data) noexcept {
    auto &slot = *static_cast<TaskSlot *>(data);
    Activation activation{Task{context}, slot};
    std::array<Entry, 1> entries;
    Stack cleanup(entries);
    const auto root = cleanup.mark();
    if (slot.cleanup != nullptr) {
        const auto reserved = cleanup.reserve();
        if (reserved.status != Status::ok ||
            cleanup.arm(reserved.token, &activation, clean, "task cleanup") != Status::ok) {
            std::abort();
        }
    }
    const auto panic = slot.body(activation.task, slot.data);
    const auto result = cleanup.unwind(root, panic.code == 0 ? Reason::complete : Reason::panic, panic);
    if (result.status != Status::ok) {
        std::abort();
    }
    slot.outcome = {panic.code == 0 ? OutcomeKind::completed : OutcomeKind::panicked, panic, {}};
}

Panic Scheduler::clean(void *data) noexcept {
    auto &activation = *static_cast<Activation *>(data);
    return activation.slot.cleanup(activation.task, activation.slot.data);
}

ScheduleStatus Scheduler::access() const noexcept {
    if (pthread_equal(worker, pthread_self()) == 0) {
        return ScheduleStatus::wrong_thread;
    }
    if (pumping || !configured) {
        return ScheduleStatus::invalid;
    }
    return ScheduleStatus::ok;
}

bool Scheduler::valid(TaskTicket ticket) const noexcept {
    return ticket.scheduler == this && ticket.index < slots.size() && ticket.id != 0 &&
           slots[ticket.index].state != TaskState::vacant && slots[ticket.index].id == ticket.id;
}

std::size_t Scheduler::runnable_count() const noexcept {
    std::size_t count = 0;
    for (const auto &slot : slots) {
        count += slot.runnable();
    }
    return count;
}

}
