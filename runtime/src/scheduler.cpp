#include "meowy/scheduler.hpp"

#include <array>
#include <cerrno>
#include <cstdlib>
#include <limits>
#include <unistd.h>

namespace meowy::prototype::v0 {

struct Scheduler::Activation final {
public:
    Task task;
    TaskSlot &slot;
};

Task::Task(Scheduler &owner, Context &value, TaskTicket identity) noexcept
    : scheduler(owner), context(value), ticket(identity) {}

ContextStatus Task::yield() noexcept {
    const auto status = scheduler.task_access(ticket);
    if (status != ScheduleStatus::ok) {
        return status == ScheduleStatus::wrong_thread ? ContextStatus::wrong_thread : ContextStatus::invalid;
    }
    return context.yield();
}

Submission Task::spawn(TaskBody body, void *data, TaskBody cleanup) noexcept {
    const auto status = scheduler.task_access(ticket);
    if (status != ScheduleStatus::ok || body == nullptr) {
        return {status == ScheduleStatus::ok ? ScheduleStatus::invalid : status, {}, {}};
    }
    return scheduler.admit(body, data, cleanup, ticket);
}

Joined Task::join(TaskTicket child) noexcept {
    return scheduler.join_child(ticket, child);
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
    return admit(body, data, cleanup, {});
}

Submission Scheduler::admit(TaskBody body, void *data, TaskBody cleanup, TaskTicket parent) noexcept {
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
        slot.owner = this;
        slot.index = index;
        slot.parent = parent;
        slot.waiting = {};
        slot.children = 0;
        if (parent.scheduler != nullptr) {
            ++slots[parent.index].children;
        }
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
        active = {this, index, slot.id};
        const auto resumed = slot.context.resume();
        active = {};
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
            wake({this, index, slot.id});
        } else if (slot.context.state() == ContextState::suspended) {
            if (slot.state != TaskState::waiting) {
                slot.state = TaskState::runnable;
            }
        } else {
            result.status = ScheduleStatus::invalid;
            break;
        }
    }
    pumping = false;
    result.runnable = runnable_count();
    result.waiting = waiting_count();
    return result;
}

TaskInfo Scheduler::inspect(TaskTicket ticket) const noexcept {
    const auto status = access();
    if (status != ScheduleStatus::ok || !valid(ticket)) {
        return {status == ScheduleStatus::ok ? ScheduleStatus::invalid : status, TaskState::vacant, {}, {}, {}, 0};
    }
    const auto &slot = slots[ticket.index];
    return {ScheduleStatus::ok, slot.state, slot.outcome, slot.parent, slot.waiting, slot.children};
}

Joined Scheduler::join(TaskTicket ticket) noexcept {
    const auto status = access();
    if (status != ScheduleStatus::ok || !valid(ticket) || slots[ticket.index].parent.scheduler != nullptr) {
        return {status == ScheduleStatus::ok ? ScheduleStatus::invalid : status, {}, {}};
    }
    return consume(ticket);
}

Joined Scheduler::join_child(TaskTicket parent, TaskTicket child) noexcept {
    const auto status = task_access(parent);
    if (status != ScheduleStatus::ok || !valid(child) || slots[child.index].parent != parent) {
        return {status == ScheduleStatus::ok ? ScheduleStatus::invalid : status, {}, {}};
    }
    auto &owner = slots[parent.index];
    while (slots[child.index].state != TaskState::settled) {
        owner.waiting = child;
        owner.state = TaskState::waiting;
        const auto result = owner.context.yield();
        owner.waiting = {};
        owner.state = TaskState::running;
        if (result != ContextStatus::ok) {
            return {ScheduleStatus::context_failed, {}, {result, {}}};
        }
    }
    return consume(child);
}

Joined Scheduler::consume(TaskTicket ticket) noexcept {
    auto &slot = slots[ticket.index];
    if (slot.state != TaskState::settled || slot.children != 0) {
        return {};
    }
    const auto result = slot.context.release();
    if (result.status != ContextStatus::ok) {
        return {ScheduleStatus::context_failed, slot.outcome, result};
    }
    const auto outcome = slot.outcome;
    if (slot.parent.scheduler != nullptr) {
        if (!valid(slot.parent) || slots[slot.parent.index].children == 0) {
            std::abort();
        }
        --slots[slot.parent.index].children;
    }
    slot.state = TaskState::vacant;
    slot.outcome = {};
    slot.body = nullptr;
    slot.cleanup = nullptr;
    slot.data = nullptr;
    slot.owner = nullptr;
    slot.parent = {};
    slot.waiting = {};
    return {ScheduleStatus::ok, outcome, {}};
}

void Scheduler::wake(TaskTicket child) noexcept {
    const auto parent = slots[child.index].parent;
    if (valid(parent)) {
        auto &owner = slots[parent.index];
        if (owner.state == TaskState::waiting && owner.waiting == child) {
            owner.state = TaskState::runnable;
        }
    }
}

void Scheduler::run(Context &context, void *data) noexcept {
    auto &slot = *static_cast<TaskSlot *>(data);
    Activation activation{Task{*slot.owner, context, {slot.owner, slot.index, slot.id}}, slot};
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
    if (slot.children != 0) {
        unjoined("body");
    }
    const auto result = cleanup.unwind(root, panic.code == 0 ? Reason::complete : Reason::panic, panic);
    if (result.status != Status::ok) {
        std::abort();
    }
    slot.outcome = {panic.code == 0 ? OutcomeKind::completed : OutcomeKind::panicked, panic, {}};
}

Panic Scheduler::clean(void *data) noexcept {
    auto &activation = *static_cast<Activation *>(data);
    const auto panic = activation.slot.cleanup(activation.task, activation.slot.data);
    if (activation.slot.children != 0) {
        unjoined("cleanup");
    }
    return panic;
}

[[noreturn]] void Scheduler::unjoined(std::string_view phase) noexcept {
    for (auto text : {std::string_view("fatal runtime protocol: "), phase,
                      std::string_view(" returned with unjoined children\n")}) {
        while (!text.empty()) {
            const auto count = ::write(STDERR_FILENO, text.data(), text.size());
            if (count < 0 && errno == EINTR) {
                continue;
            }
            if (count <= 0) {
                break;
            }
            text.remove_prefix(static_cast<std::size_t>(count));
        }
    }
    std::abort();
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

ScheduleStatus Scheduler::task_access(TaskTicket ticket) const noexcept {
    if (pthread_equal(worker, pthread_self()) == 0) {
        return ScheduleStatus::wrong_thread;
    }
    if (!pumping || active != ticket || !valid(ticket) || slots[ticket.index].state != TaskState::running) {
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

std::size_t Scheduler::waiting_count() const noexcept {
    std::size_t count = 0;
    for (const auto &slot : slots) {
        count += slot.state == TaskState::waiting;
    }
    return count;
}

}
