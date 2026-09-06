#include "meowy/scheduler.hpp"

#include <algorithm>
#include <array>
#include <cerrno>
#include <cstdio>
#include <cstdlib>
#include <limits>
#include <source_location>
#include <string_view>
#include <sys/mman.h>
#include <sys/resource.h>
#include <unistd.h>

using namespace meowy::prototype::v0;

namespace {

constexpr std::size_t stack_bytes = 262144;
bool fail_map = false;
bool fail_protect = false;
bool fail_release = false;

void check(bool value, std::source_location site = std::source_location::current()) {
    if (!value) {
        std::fprintf(stderr, "check failed at %s:%u\n", site.file_name(), site.line());
        std::abort();
    }
}

struct Work final {
public:
    int steps = 0;
    int yields = 0;
    int cleanups = 0;
    bool cleanup_yields = false;
    bool cleaned = false;
    Panic panic;

    static Panic run(Task &task, void *data) noexcept {
        auto &work = *static_cast<Work *>(data);
        for (int index = 0; index < work.yields; ++index) {
            ++work.steps;
            check(task.yield() == ContextStatus::ok);
        }
        ++work.steps;
        return work.panic;
    }

    static Panic clean(Task &task, void *data) noexcept {
        auto &work = *static_cast<Work *>(data);
        ++work.cleanups;
        if (work.cleanup_yields) {
            check(task.yield() == ContextStatus::ok);
        }
        work.cleaned = true;
        return {};
    }
};

void invalid_configuration_and_empty_queue() {
    std::array<TaskSlot, 0> none;
    Scheduler empty(none, stack_bytes);
    Work work;
    check(empty.submit(Work::run, &work).status == ScheduleStatus::invalid);
    std::array<TaskSlot, 1> slots;
    Scheduler invalid(slots, 0);
    check(invalid.submit(Work::run, &work).status == ScheduleStatus::invalid);
    std::array<TaskSlot, 1> valid_slots;
    Scheduler scheduler(valid_slots, stack_bytes);
    check(scheduler.submit(nullptr, nullptr).status == ScheduleStatus::invalid);
    const auto pumped = scheduler.pump(5);
    check(pumped.status == ScheduleStatus::ok && pumped.resumed == 0 && pumped.runnable == 0);
    check(scheduler.join({}).status == ScheduleStatus::invalid);
    check(scheduler.inspect({}).status == ScheduleStatus::invalid);
}

void round_robin_and_bounded_pump() {
    std::array<TaskSlot, 3> slots;
    Scheduler scheduler(slots, stack_bytes);
    std::array<Work, 3> work;
    std::array<TaskTicket, 3> tickets;
    for (std::size_t index = 0; index < work.size(); ++index) {
        work[index].yields = 3;
        const auto submitted = scheduler.submit(Work::run, &work[index], Work::clean);
        check(submitted.status == ScheduleStatus::ok);
        tickets[index] = submitted.ticket;
    }
    const auto idle = scheduler.pump(0);
    check(idle.status == ScheduleStatus::ok && idle.resumed == 0 && idle.runnable == 3);
    for (int step = 0; step < 3; ++step) {
        const auto pumped = scheduler.pump(3);
        check(pumped.status == ScheduleStatus::ok && pumped.resumed == 3 && pumped.runnable == 3);
        for (const auto &value : work) {
            check(value.steps == step + 1 && !value.cleaned);
        }
    }
    const auto finished = scheduler.pump(20);
    check(finished.status == ScheduleStatus::ok && finished.resumed == 3 && finished.runnable == 0);
    for (std::size_t index = 0; index < work.size(); ++index) {
        check(work[index].steps == 4 && work[index].cleanups == 1 && work[index].cleaned);
        check(scheduler.join(tickets[index]).status == ScheduleStatus::ok);
    }
}

void selector_handles_empty_settled_and_wrapped_cursors() {
    check(select_task({}, std::numeric_limits<std::size_t>::max()) == 0);
    std::array<TaskSlot, 3> slots;
    Scheduler scheduler(slots, stack_bytes);
    Work first;
    Work second;
    const auto a = scheduler.submit(Work::run, &first);
    const auto b = scheduler.submit(Work::run, &second);
    check(a.status == ScheduleStatus::ok && b.status == ScheduleStatus::ok);
    check(select_task(slots, 5) == 0);
    check(select_task(slots, 4) == 1);
    check(scheduler.pump(1).resumed == 1);
    check(select_task(slots, 0) == 1);
    check(select_task(slots, std::numeric_limits<std::size_t>::max()) == 1);
    check(scheduler.pump(1).resumed == 1);
    check(select_task(slots, 1) == slots.size());
    check(scheduler.join(a.ticket).status == ScheduleStatus::ok);
    check(scheduler.join(b.ticket).status == ScheduleStatus::ok);
}

void settled_tasks_hold_capacity_until_join() {
    std::array<TaskSlot, 1> slots;
    Scheduler scheduler(slots, stack_bytes);
    Work first;
    Work second;
    const auto a = scheduler.submit(Work::run, &first, Work::clean);
    check(a.status == ScheduleStatus::ok);
    check(scheduler.submit(Work::run, &second).status == ScheduleStatus::full);
    check(scheduler.join(a.ticket).status == ScheduleStatus::invalid);
    check(scheduler.pump(1).resumed == 1);
    check(scheduler.submit(Work::run, &second).status == ScheduleStatus::full && second.steps == 0);
    const auto joined = scheduler.join(a.ticket);
    check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::completed);
    check(first.cleaned);
    check(scheduler.join(a.ticket).status == ScheduleStatus::invalid);
    const auto b = scheduler.submit(Work::run, &second);
    check(b.status == ScheduleStatus::ok && a.ticket.index == b.ticket.index && a.ticket.id != b.ticket.id);
    check(scheduler.inspect(a.ticket).status == ScheduleStatus::invalid);
    check(scheduler.pump(1).resumed == 1);
    check(scheduler.join(b.ticket).status == ScheduleStatus::ok);
}

void panic_settles_only_after_cleanup() {
    std::array<TaskSlot, 1> slots;
    Scheduler scheduler(slots, stack_bytes);
    Work work;
    work.panic = {6, "body failed"};
    work.cleanup_yields = true;
    const auto submitted = scheduler.submit(Work::run, &work, Work::clean);
    check(submitted.status == ScheduleStatus::ok);
    const auto first = scheduler.pump(1);
    check(first.resumed == 1 && first.runnable == 1 && work.cleanups == 1 && !work.cleaned);
    const auto waiting = scheduler.inspect(submitted.ticket);
    check(waiting.status == ScheduleStatus::ok && waiting.state == TaskState::runnable);
    check(waiting.outcome.kind == OutcomeKind::pending);
    check(scheduler.join(submitted.ticket).status == ScheduleStatus::invalid);
    check(scheduler.pump(1).resumed == 1 && work.cleaned);
    const auto joined = scheduler.join(submitted.ticket);
    check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::panicked);
    check(joined.outcome.panic.code == 6 && joined.outcome.panic.message() == "body failed");
}

void tickets_reject_other_schedulers_and_invalid_identity() {
    std::array<TaskSlot, 1> slots;
    std::array<TaskSlot, 1> other_slots;
    Scheduler scheduler(slots, stack_bytes);
    Scheduler other(other_slots, stack_bytes);
    Work work;
    const auto submitted = scheduler.submit(Work::run, &work);
    check(submitted.status == ScheduleStatus::ok);
    check(other.inspect(submitted.ticket).status == ScheduleStatus::invalid);
    check(other.join(submitted.ticket).status == ScheduleStatus::invalid);
    check(scheduler.inspect({&scheduler, 9, submitted.ticket.id}).status == ScheduleStatus::invalid);
    check(scheduler.inspect({&scheduler, 0, 0}).status == ScheduleStatus::invalid);
    check(scheduler.pump(1).resumed == 1);
    check(scheduler.join(submitted.ticket).status == ScheduleStatus::ok);
}

struct ThreadCall final {
public:
    Scheduler &scheduler;
    TaskTicket ticket;
    Work &work;

    static void *run(void *data) noexcept {
        auto &value = *static_cast<ThreadCall *>(data);
        check(value.scheduler.submit(Work::run, &value.work).status == ScheduleStatus::wrong_thread);
        check(value.scheduler.pump(1).status == ScheduleStatus::wrong_thread);
        check(value.scheduler.inspect(value.ticket).status == ScheduleStatus::wrong_thread);
        check(value.scheduler.join(value.ticket).status == ScheduleStatus::wrong_thread);
        return nullptr;
    }
};

void worker_affinity_rejects_foreign_access() {
    std::array<TaskSlot, 1> slots;
    Scheduler scheduler(slots, stack_bytes);
    Work work;
    const auto submitted = scheduler.submit(Work::run, &work);
    check(submitted.status == ScheduleStatus::ok);
    ThreadCall call{scheduler, submitted.ticket, work};
    pthread_t thread{};
    check(pthread_create(&thread, nullptr, ThreadCall::run, &call) == 0);
    check(pthread_join(thread, nullptr) == 0 && work.steps == 0);
    check(scheduler.pump(1).resumed == 1);
    check(scheduler.join(submitted.ticket).status == ScheduleStatus::ok);
}

struct Reentry final {
public:
    Scheduler &scheduler;
    TaskTicket ticket;

    static Panic run(Task &task, void *data) noexcept {
        auto &value = *static_cast<Reentry *>(data);
        check(value.scheduler.submit(run, data).status == ScheduleStatus::invalid);
        check(value.scheduler.pump(1).status == ScheduleStatus::invalid);
        check(value.scheduler.inspect(value.ticket).status == ScheduleStatus::invalid);
        check(value.scheduler.join(value.ticket).status == ScheduleStatus::invalid);
        check(task.yield() == ContextStatus::ok);
        return {};
    }
};

void task_callbacks_cannot_reenter_scheduler() {
    std::array<TaskSlot, 1> slots;
    Scheduler scheduler(slots, stack_bytes);
    Reentry value{scheduler, {}};
    const auto submitted = scheduler.submit(Reentry::run, &value);
    check(submitted.status == ScheduleStatus::ok);
    value.ticket = submitted.ticket;
    check(scheduler.pump(2).resumed == 2);
    check(scheduler.join(submitted.ticket).status == ScheduleStatus::ok);
}

void failed_admission_is_joinable_without_running_callbacks() {
    std::array<TaskSlot, 1> slots;
    Scheduler scheduler(slots, stack_bytes);
    Work work;
    fail_map = true;
    const auto submitted = scheduler.submit(Work::run, &work, Work::clean);
    fail_map = false;
    check(submitted.status == ScheduleStatus::context_failed);
    check(submitted.context.memory.status == MemoryStatus::allocation_failed && submitted.context.memory.error == ENOMEM);
    check(scheduler.inspect(submitted.ticket).state == TaskState::settled);
    check(scheduler.submit(Work::run, &work).status == ScheduleStatus::full);
    check(scheduler.pump(2).resumed == 0 && work.steps == 0 && work.cleanups == 0);
    const auto joined = scheduler.join(submitted.ticket);
    check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::spawn_failed);
}

void failed_admission_retains_rollback_mapping_until_join() {
    std::array<TaskSlot, 1> slots;
    Scheduler scheduler(slots, stack_bytes);
    Work work;
    fail_protect = true;
    fail_release = true;
    const auto submitted = scheduler.submit(Work::run, &work, Work::clean);
    fail_protect = false;
    check(submitted.status == ScheduleStatus::context_failed);
    check(submitted.context.memory.status == MemoryStatus::protection_failed);
    check(submitted.context.memory.rollback_error == EIO);
    check(scheduler.join(submitted.ticket).status == ScheduleStatus::context_failed);
    check(scheduler.submit(Work::run, &work).status == ScheduleStatus::full);
    fail_release = false;
    check(scheduler.join(submitted.ticket).status == ScheduleStatus::ok);
    check(work.steps == 0 && work.cleanups == 0);
}

void failed_join_preserves_settled_ticket() {
    std::array<TaskSlot, 1> slots;
    Scheduler scheduler(slots, stack_bytes);
    Work work;
    const auto submitted = scheduler.submit(Work::run, &work, Work::clean);
    check(submitted.status == ScheduleStatus::ok && scheduler.pump(1).resumed == 1);
    fail_release = true;
    const auto joined = scheduler.join(submitted.ticket);
    fail_release = false;
    check(joined.status == ScheduleStatus::context_failed && joined.context.memory.error == EIO);
    check(scheduler.inspect(submitted.ticket).state == TaskState::settled);
    check(scheduler.join(submitted.ticket).status == ScheduleStatus::ok && work.cleanups == 1);
}

struct Family final {
public:
    TaskTicket child;
    int result = 0;
    bool child_cleaned = false;
    bool parent_cleaned = false;

    struct Borrow final {
    public:
        int &value;
        Family &family;
    };

    static Panic child_body(Task &task, void *data) noexcept {
        auto &borrow = *static_cast<Borrow *>(data);
        check(borrow.value == 41);
        ++borrow.value;
        check(task.yield() == ContextStatus::ok);
        check(borrow.value == 42);
        ++borrow.value;
        return {};
    }

    static Panic child_cleanup(Task &task, void *data) noexcept {
        auto &borrow = *static_cast<Borrow *>(data);
        check(borrow.value == 43);
        check(task.yield() == ContextStatus::ok);
        borrow.family.child_cleaned = true;
        return {};
    }

    static Panic parent_body(Task &task, void *data) noexcept {
        auto &family = *static_cast<Family *>(data);
        int local = 41;
        Borrow borrow{local, family};
        const auto submitted = task.spawn(child_body, &borrow, child_cleanup);
        check(submitted.status == ScheduleStatus::ok);
        family.child = submitted.ticket;
        const auto joined = task.join(submitted.ticket);
        check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::completed);
        check(local == 43 && family.child_cleaned);
        family.result = local;
        check(task.join(submitted.ticket).status == ScheduleStatus::invalid);
        return {};
    }

    static Panic parent_cleanup(Task &, void *data) noexcept {
        auto &family = *static_cast<Family *>(data);
        check(family.child_cleaned && family.result == 43);
        family.parent_cleaned = true;
        return {};
    }
};

void waiting_parent_keeps_locals_alive_and_releases_worker() {
    std::array<TaskSlot, 3> slots;
    Scheduler scheduler(slots, stack_bytes);
    Family family;
    Work other;
    const auto parent = scheduler.submit(Family::parent_body, &family, Family::parent_cleanup);
    const auto unrelated = scheduler.submit(Work::run, &other);
    check(parent.status == ScheduleStatus::ok && unrelated.status == ScheduleStatus::ok);
    const auto first = scheduler.pump(1);
    check(first.status == ScheduleStatus::ok && first.resumed == 1 && first.waiting == 1 && first.runnable == 2);
    const auto waiting = scheduler.inspect(parent.ticket);
    check(waiting.state == TaskState::waiting && waiting.children == 1 && waiting.waiting == family.child);
    check(scheduler.inspect(family.child).parent == parent.ticket);
    check(scheduler.join(family.child).status == ScheduleStatus::invalid);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::invalid);
    check(scheduler.pump(1).resumed == 1 && other.steps == 1 && family.result == 0);
    const auto rest = scheduler.pump(10);
    check(rest.status == ScheduleStatus::ok && rest.runnable == 0 && rest.waiting == 0);
    check(family.parent_cleaned && family.result == 43);
    check(scheduler.inspect(family.child).status == ScheduleStatus::invalid);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
    check(scheduler.join(unrelated.ticket).status == ScheduleStatus::ok);
}

struct Chain final {
public:
    int depth;
    int &result;

    static Panic run(Task &task, void *data) noexcept {
        auto &chain = *static_cast<Chain *>(data);
        int local = 1;
        if (chain.depth != 0) {
            Chain nested{chain.depth - 1, local};
            const auto child = task.spawn(run, &nested);
            check(child.status == ScheduleStatus::ok);
            check(task.join(child.ticket).status == ScheduleStatus::ok);
        }
        check(local == chain.depth + 1);
        chain.result = local + 1;
        return {};
    }
};

void nested_children_use_fixed_slots_and_join_each_parent() {
    std::array<TaskSlot, 5> slots;
    Scheduler scheduler(slots, stack_bytes);
    int result = 0;
    Chain chain{4, result};
    const auto parent = scheduler.submit(Chain::run, &chain);
    check(parent.status == ScheduleStatus::ok);
    const auto pumped = scheduler.pump(20);
    check(pumped.status == ScheduleStatus::ok && pumped.resumed == 9 && pumped.waiting == 0);
    check(result == 6 && scheduler.inspect(parent.ticket).children == 0);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
}

struct CleanupChild final {
public:
    int result = 0;

    static Panic body(Task &, void *) noexcept { return {}; }

    static Panic child(Task &task, void *data) noexcept {
        auto &value = *static_cast<int *>(data);
        check(value == 5);
        check(task.yield() == ContextStatus::ok);
        ++value;
        return {};
    }

    static Panic clean(Task &task, void *data) noexcept {
        int local = 5;
        const auto submitted = task.spawn(child, &local);
        check(submitted.status == ScheduleStatus::ok);
        check(task.join(submitted.ticket).status == ScheduleStatus::ok);
        static_cast<CleanupChild *>(data)->result = local;
        return {};
    }
};

void parent_cleanup_can_spawn_and_wait_for_its_child() {
    std::array<TaskSlot, 2> slots;
    Scheduler scheduler(slots, stack_bytes);
    CleanupChild value;
    const auto parent = scheduler.submit(CleanupChild::body, &value, CleanupChild::clean);
    check(parent.status == ScheduleStatus::ok);
    check(scheduler.pump(1).waiting == 1);
    check(scheduler.inspect(parent.ticket).outcome.kind == OutcomeKind::pending);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::invalid);
    check(scheduler.pump(5).waiting == 0 && value.result == 6);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
}

struct ChildFailures final {
public:
    bool done = false;
    TaskTicket child;

    static Panic full(Task &task, void *data) noexcept {
        Work work;
        const auto child = task.spawn(Work::run, &work, Work::clean);
        check(child.status == ScheduleStatus::full && child.ticket.scheduler == nullptr);
        check(task.join(child.ticket).status == ScheduleStatus::invalid);
        check(work.steps == 0 && work.cleanups == 0);
        static_cast<ChildFailures *>(data)->done = true;
        return {};
    }

    static Panic admission(Task &task, void *data) noexcept {
        Work work;
        fail_map = true;
        const auto failed = task.spawn(Work::run, &work, Work::clean);
        fail_map = false;
        check(failed.status == ScheduleStatus::context_failed && failed.context.memory.error == ENOMEM);
        const auto result = task.join(failed.ticket);
        check(result.status == ScheduleStatus::ok && result.outcome.kind == OutcomeKind::spawn_failed);
        check(work.steps == 0 && work.cleanups == 0);
        work.panic = {6, "child failed"};
        work.cleanup_yields = true;
        const auto child = task.spawn(Work::run, &work, Work::clean);
        check(child.status == ScheduleStatus::ok && child.ticket.id != failed.ticket.id);
        check(task.join(failed.ticket).status == ScheduleStatus::invalid);
        const auto joined = task.join(child.ticket);
        check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::panicked && work.cleaned);
        check(joined.outcome.panic.message() == "child failed");
        static_cast<ChildFailures *>(data)->done = true;
        return {};
    }

    static Panic release(Task &task, void *data) noexcept {
        auto &value = *static_cast<ChildFailures *>(data);
        Work work;
        const auto child = task.spawn(Work::run, &work, Work::clean);
        check(child.status == ScheduleStatus::ok);
        value.child = child.ticket;
        fail_release = true;
        const auto failed = task.join(child.ticket);
        fail_release = false;
        check(failed.status == ScheduleStatus::context_failed && failed.context.memory.error == EIO);
        check(work.cleaned && work.cleanups == 1);
        check(task.yield() == ContextStatus::ok);
        check(task.join(child.ticket).status == ScheduleStatus::ok && work.cleanups == 1);
        value.done = true;
        return {};
    }

    static Panic rollback(Task &task, void *data) noexcept {
        Work work;
        fail_protect = true;
        fail_release = true;
        const auto child = task.spawn(Work::run, &work, Work::clean);
        fail_protect = false;
        check(child.status == ScheduleStatus::context_failed && child.context.memory.rollback_error == EIO);
        check(task.join(child.ticket).status == ScheduleStatus::context_failed);
        fail_release = false;
        const auto joined = task.join(child.ticket);
        check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::spawn_failed);
        check(work.steps == 0 && work.cleanups == 0);
        static_cast<ChildFailures *>(data)->done = true;
        return {};
    }
};

void child_admission_obeys_parent_capacity_and_failure_ownership() {
    for (const auto body : {ChildFailures::full, ChildFailures::admission, ChildFailures::rollback}) {
        std::array<TaskSlot, 2> slots;
        const auto count = body == ChildFailures::full ? 1 : 2;
        Scheduler scheduler(std::span(slots).first(count), stack_bytes);
        ChildFailures value;
        const auto parent = scheduler.submit(body, &value);
        check(parent.status == ScheduleStatus::ok);
        check(scheduler.pump(10).status == ScheduleStatus::ok && value.done);
        check(scheduler.inspect(parent.ticket).children == 0);
        check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
    }
}

void failed_child_release_retains_parent_ownership_until_retry() {
    std::array<TaskSlot, 2> slots;
    Scheduler scheduler(slots, stack_bytes);
    ChildFailures value;
    const auto parent = scheduler.submit(ChildFailures::release, &value);
    check(parent.status == ScheduleStatus::ok);
    check(scheduler.pump(3).resumed == 3);
    check(!value.done && scheduler.inspect(parent.ticket).children == 1);
    check(scheduler.inspect(value.child).state == TaskState::settled);
    check(scheduler.join(value.child).status == ScheduleStatus::invalid);
    check(scheduler.pump(1).resumed == 1 && value.done);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
}

struct Capabilities final {
public:
    Task *first = nullptr;
    Task *second = nullptr;
    TaskTicket first_root;
    TaskTicket second_root;
    TaskTicket first_child;
    TaskTicket second_child;
    TaskTicket foreign;
    bool checked = false;

    static Panic inspect(Task &task, void *data) noexcept {
        auto &value = *static_cast<Capabilities *>(data);
        Work work;
        check(value.first != nullptr && value.second != nullptr);
        for (auto *owner : {value.first, value.second}) {
            check(owner->spawn(Work::run, &work).status == ScheduleStatus::invalid);
            check(owner->join(value.first_child).status == ScheduleStatus::invalid);
            check(owner->yield() == ContextStatus::invalid);
        }
        for (const auto ticket : {value.first_root, value.second_root, value.first_child, value.second_child, value.foreign}) {
            check(task.join(ticket).status == ScheduleStatus::invalid);
        }
        value.checked = true;
        return {};
    }

    static Panic first_body(Task &task, void *data) noexcept {
        auto &value = *static_cast<Capabilities *>(data);
        value.first = &task;
        const auto child = task.spawn(inspect, &value);
        check(child.status == ScheduleStatus::ok);
        value.first_child = child.ticket;
        check(task.join(child.ticket).status == ScheduleStatus::ok);
        value.first = nullptr;
        return {};
    }

    static Panic second_body(Task &task, void *data) noexcept {
        auto &value = *static_cast<Capabilities *>(data);
        value.second = &task;
        Work work;
        const auto child = task.spawn(Work::run, &work);
        check(child.status == ScheduleStatus::ok);
        value.second_child = child.ticket;
        check(task.join(child.ticket).status == ScheduleStatus::ok);
        value.second = nullptr;
        return {};
    }
};

void child_apis_reject_parent_sibling_root_and_foreign_capabilities() {
    std::array<TaskSlot, 4> slots;
    std::array<TaskSlot, 1> other;
    Scheduler scheduler(slots, stack_bytes);
    Scheduler foreign(other, stack_bytes);
    Capabilities value;
    Work work;
    const auto external = foreign.submit(Work::run, &work);
    value.foreign = external.ticket;
    const auto first = scheduler.submit(Capabilities::first_body, &value);
    const auto second = scheduler.submit(Capabilities::second_body, &value);
    check(first.status == ScheduleStatus::ok && second.status == ScheduleStatus::ok);
    value.first_root = first.ticket;
    value.second_root = second.ticket;
    check(scheduler.pump(20).status == ScheduleStatus::ok && value.checked);
    check(scheduler.join(first.ticket).status == ScheduleStatus::ok);
    check(scheduler.join(second.ticket).status == ScheduleStatus::ok);
    check(foreign.pump(1).resumed == 1);
    check(foreign.join(external.ticket).status == ScheduleStatus::ok);
}

struct WaitOrder final {
public:
    TaskTicket first;
    TaskTicket second;
    bool resumed = false;

    static Panic parent(Task &task, void *data) noexcept {
        auto &order = *static_cast<WaitOrder *>(data);
        Work slow;
        slow.yields = 2;
        Work fast;
        const auto a = task.spawn(Work::run, &slow);
        const auto b = task.spawn(Work::run, &fast);
        check(a.status == ScheduleStatus::ok && b.status == ScheduleStatus::ok);
        order.first = a.ticket;
        order.second = b.ticket;
        check(task.join(a.ticket).status == ScheduleStatus::ok);
        order.resumed = true;
        check(task.join(b.ticket).status == ScheduleStatus::ok);
        return {};
    }
};

void only_the_selected_child_wakes_its_waiting_parent() {
    std::array<TaskSlot, 3> slots;
    Scheduler scheduler(slots, stack_bytes);
    WaitOrder value;
    const auto parent = scheduler.submit(WaitOrder::parent, &value);
    check(parent.status == ScheduleStatus::ok);
    check(scheduler.pump(3).resumed == 3);
    check(scheduler.inspect(value.second).state == TaskState::settled);
    const auto waiting = scheduler.inspect(parent.ticket);
    check(waiting.state == TaskState::waiting && waiting.waiting == value.first && !value.resumed);
    check(scheduler.pump(10).waiting == 0 && value.resumed);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
}

Panic unjoined_child(Task &, void *) noexcept {
    constexpr std::string_view text = "child must not run after callback return\n";
    static_cast<void>(::write(STDERR_FILENO, text.data(), text.size()));
    return {};
}

Panic leave_child(Task &task, void *) noexcept {
    int local = 42;
    check(task.spawn(unjoined_child, &local).status == ScheduleStatus::ok);
    return {};
}

struct ScopedWork final {
public:
    bool closed = false;
    bool done = false;

    static Panic run(Task &task, void *data) noexcept {
        auto &value = *static_cast<ScopedWork *>(data);
        Work before;
        before.yields = 1;
        const auto older = task.spawn(Work::run, &before, Work::clean);
        check(older.status == ScheduleStatus::ok);
        const auto scope = task.mark();
        check(scope.status == ScheduleStatus::ok);
        {
            Work local;
            local.yields = 1;
            local.cleanup_yields = true;
            check(task.spawn(Work::run, &local, Work::clean).status == ScheduleStatus::ok);
            const auto closed = task.close(scope.mark);
            check(closed.status == ScheduleStatus::ok && closed.joined == 1 && closed.panicked == 0);
            check(local.cleaned && local.cleanups == 1 && local.steps == 2);
        }
        value.closed = true;
        check(task.yield() == ContextStatus::ok);
        check(task.join(older.ticket).status == ScheduleStatus::ok && before.cleaned);
        value.done = true;
        return {};
    }
};

void closing_scope_waits_while_locals_live_and_preserves_older_children() {
    for (const bool cleanup : {false, true}) {
        std::array<TaskSlot, 3> slots;
        Scheduler scheduler(slots, stack_bytes);
        ScopedWork value;
        const auto parent = scheduler.submit(cleanup ? CleanupChild::body : ScopedWork::run, &value,
                                             cleanup ? ScopedWork::run : nullptr);
        check(parent.status == ScheduleStatus::ok);
        for (int step = 0; step < 16 && !value.closed; ++step) {
            check(scheduler.pump(1).status == ScheduleStatus::ok);
        }
        check(value.closed && !value.done);
        const auto state = scheduler.inspect(parent.ticket);
        check(state.scopes == 0 && state.children == 1);
        check(scheduler.pump(5).status == ScheduleStatus::ok && value.done);
        check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
    }
}

struct ScopeNesting final {
public:
    ScopeMark old;

    static Panic first(Task &task, void *data) noexcept {
        const auto outer = task.mark();
        Work a;
        Work b;
        check(outer.status == ScheduleStatus::ok);
        check(task.spawn(Work::run, &a).status == ScheduleStatus::ok);
        const auto inner = task.mark();
        check(inner.status == ScheduleStatus::ok);
        check(task.spawn(Work::run, &b).status == ScheduleStatus::ok);
        check(task.close(outer.mark).status == ScheduleStatus::invalid);
        const auto inner_done = task.close(inner.mark);
        check(inner_done.status == ScheduleStatus::ok && inner_done.joined == 1 && b.steps == 1);
        check(task.close(inner.mark).status == ScheduleStatus::invalid);
        const auto outer_done = task.close(outer.mark);
        check(outer_done.status == ScheduleStatus::ok && outer_done.joined == 1 && a.steps == 1);
        static_cast<ScopeNesting *>(data)->old = outer.mark;
        std::array<ScopeMark, Task::scope_limit> marks;
        for (auto &mark : marks) {
            const auto opened = task.mark();
            check(opened.status == ScheduleStatus::ok);
            mark = opened.mark;
        }
        check(task.mark().status == ScheduleStatus::full);
        check(task.close(marks.front()).status == ScheduleStatus::invalid);
        for (std::size_t index = marks.size(); index != 0; --index) {
            const auto closed = task.close(marks[index - 1]);
            check(closed.status == ScheduleStatus::ok && closed.joined == 0);
        }
        check(task.close({}).status == ScheduleStatus::invalid);
        return {};
    }

    static Panic reused(Task &task, void *data) noexcept {
        const auto opened = task.mark();
        check(opened.status == ScheduleStatus::ok);
        check(task.close(static_cast<ScopeNesting *>(data)->old).status == ScheduleStatus::invalid);
        check(task.close(opened.mark).status == ScheduleStatus::ok);
        return {};
    }
};

void scope_nesting_capacity_and_generation_are_checked() {
    std::array<TaskSlot, 3> slots;
    Scheduler scheduler(slots, stack_bytes);
    ScopeNesting value;
    for (const auto body : {ScopeNesting::first, ScopeNesting::reused}) {
        const auto parent = scheduler.submit(body, &value);
        check(parent.status == ScheduleStatus::ok);
        check(scheduler.pump(20).status == ScheduleStatus::ok);
        check(scheduler.inspect(parent.ticket).scopes == 0);
        check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
    }
}

struct ScopeAccess final {
public:
    Task *first = nullptr;
    ScopeMark first_mark;
    ScopeMark second_mark;
    bool checked = false;

    static Panic a(Task &task, void *data) noexcept {
        auto &value = *static_cast<ScopeAccess *>(data);
        const auto opened = task.mark();
        check(opened.status == ScheduleStatus::ok);
        value.first = &task;
        value.first_mark = opened.mark;
        check(task.yield() == ContextStatus::ok);
        check(task.close(value.second_mark).status == ScheduleStatus::invalid);
        check(task.close(opened.mark).status == ScheduleStatus::ok);
        value.first = nullptr;
        return {};
    }

    static Panic b(Task &task, void *data) noexcept {
        auto &value = *static_cast<ScopeAccess *>(data);
        const auto opened = task.mark();
        check(opened.status == ScheduleStatus::ok && value.first != nullptr);
        value.second_mark = opened.mark;
        check(value.first->mark().status == ScheduleStatus::invalid);
        check(value.first->close(value.first_mark).status == ScheduleStatus::invalid);
        check(task.close(value.first_mark).status == ScheduleStatus::invalid);
        std::array<ChildFailure, 1> failures;
        failures[0].ticket.id = 919;
        check(value.first->close(value.first_mark, failures).status == ScheduleStatus::invalid);
        check(task.close(value.first_mark, failures).status == ScheduleStatus::invalid);
        check(failures[0].ticket.id == 919);
        check(task.close(opened.mark).status == ScheduleStatus::ok);
        value.checked = true;
        return {};
    }

    static void *foreign(void *data) noexcept {
        auto &value = *static_cast<ScopeAccess *>(data);
        check(value.first->mark().status == ScheduleStatus::wrong_thread);
        check(value.first->close(value.first_mark).status == ScheduleStatus::wrong_thread);
        std::array<ChildFailure, 1> failures;
        failures[0].ticket.id = 919;
        check(value.first->close(value.first_mark, failures).status == ScheduleStatus::wrong_thread);
        check(failures[0].ticket.id == 919);
        return nullptr;
    }
};

void scope_operations_require_active_owner_and_worker() {
    std::array<TaskSlot, 2> slots;
    Scheduler scheduler(slots, stack_bytes);
    ScopeAccess value;
    const auto a = scheduler.submit(ScopeAccess::a, &value);
    const auto b = scheduler.submit(ScopeAccess::b, &value);
    check(a.status == ScheduleStatus::ok && b.status == ScheduleStatus::ok);
    check(scheduler.pump(1).resumed == 1);
    pthread_t thread{};
    check(pthread_create(&thread, nullptr, ScopeAccess::foreign, &value) == 0);
    check(pthread_join(thread, nullptr) == 0);
    check(scheduler.pump(5).status == ScheduleStatus::ok && value.checked);
    check(scheduler.join(a.ticket).status == ScheduleStatus::ok);
    check(scheduler.join(b.ticket).status == ScheduleStatus::ok);
}

Panic scope_failures(Task &task, void *) noexcept {
    const auto opened = task.mark();
    check(opened.status == ScheduleStatus::ok);
    Work normal;
    Work first;
    Work second;
    Work failed;
    first.panic = {6, "first failed"};
    second.panic = {2, "second failed"};
    for (auto *work : {&normal, &first, &second}) {
        check(task.spawn(Work::run, work, Work::clean).status == ScheduleStatus::ok);
    }
    fail_map = true;
    const auto refused = task.spawn(Work::run, &failed, Work::clean);
    fail_map = false;
    check(refused.status == ScheduleStatus::context_failed);
    check(task.spawn(Work::run, &failed).status == ScheduleStatus::full);
    const auto closed = task.close(opened.mark);
    check(closed.status == ScheduleStatus::ok && closed.joined == 4 && closed.panicked == 2 && closed.spawn_failed == 1);
    check(closed.first_failure.kind == OutcomeKind::panicked && closed.first_failure.panic.message() == "first failed");
    check(normal.cleaned && first.cleaned && second.cleaned && failed.steps == 0 && failed.cleanups == 0);
    check(task.close(opened.mark).status == ScheduleStatus::invalid);
    return {};
}

void scope_close_reports_child_failures_without_hiding_later_children() {
    std::array<TaskSlot, 5> slots;
    Scheduler scheduler(slots, stack_bytes);
    const auto parent = scheduler.submit(scope_failures, nullptr);
    check(parent.status == ScheduleStatus::ok && scheduler.pump(20).status == ScheduleStatus::ok);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
}

struct FailureBatches final {
public:
    std::array<ChildFailure, 3> seen;
    bool done = false;

    static Panic run(Task &task, void *data) noexcept {
        auto &value = *static_cast<FailureBatches *>(data);
        const auto scope = task.mark();
        check(scope.status == ScheduleStatus::ok);
        std::array<Work, 7> work;
        char first_text[] = "batch first";
        char second_text[] = "batch second";
        work[1].panic = {6, {first_text, sizeof(first_text) - 1}};
        work[3].panic = {2, {second_text, sizeof(second_text) - 1}};
        std::fill_n(first_text, sizeof(first_text), 'x');
        std::fill_n(second_text, sizeof(second_text), 'y');
        std::array<TaskTicket, 5> tickets;
        for (std::size_t index = 0; index < 4; ++index) {
            const auto child = task.spawn(Work::run, &work[index], Work::clean);
            check(child.status == ScheduleStatus::ok);
            tickets[index] = child.ticket;
        }
        fail_map = true;
        const auto refused = task.spawn(Work::run, &work[4], Work::clean);
        fail_map = false;
        check(refused.status == ScheduleStatus::context_failed);
        tickets[4] = refused.ticket;
        std::array<ChildFailure, 1> batch;
        const auto first = task.close(scope.mark, batch);
        check(first.status == ScheduleStatus::report_full && first.reported == 1 && first.joined == 3 && first.panicked == 1);
        check(first.pending == tickets[3] && first.pending_result.status == ScheduleStatus::report_full);
        check(first.pending_result.outcome.panic.message() == "batch second");
        value.seen[0] = batch[0];
        work[1].panic = {};
        check(value.seen[0].ticket == tickets[1] && value.seen[0].outcome.panic.message() == "batch first");
        const auto extra = task.spawn(Work::run, &work[5], Work::clean);
        const auto reused = task.spawn(Work::run, &work[6], Work::clean);
        check(extra.status == ScheduleStatus::ok && reused.status == ScheduleStatus::ok);
        check(reused.ticket.index == value.seen[0].ticket.index && reused.ticket.id != value.seen[0].ticket.id);
        check(task.join(value.seen[0].ticket).status == ScheduleStatus::invalid);
        const auto second = task.close(scope.mark, batch);
        check(second.status == ScheduleStatus::report_full && second.reported == 1 && second.joined == 4 && second.panicked == 2);
        check(second.pending == tickets[4] && second.spawn_failed == 0);
        value.seen[1] = batch[0];
        work[3].panic = {};
        check(value.seen[1].ticket == tickets[3] && value.seen[1].outcome.panic.message() == "batch second");
        const auto third = task.close(scope.mark, batch);
        check(third.status == ScheduleStatus::ok && third.reported == 1 && third.joined == 7);
        check(third.panicked == 2 && third.spawn_failed == 1 && third.first_failure.panic.message() == "batch first");
        value.seen[2] = batch[0];
        check(value.seen[2].ticket == tickets[4] && value.seen[2].outcome.kind == OutcomeKind::spawn_failed);
        check(value.seen[2].outcome.context.memory.error == ENOMEM);
        check(work[4].steps == 0 && work[4].cleanups == 0);
        for (const auto index : {0, 1, 2, 3, 5, 6}) { check(work[index].cleanups == 1); }
        check(task.close(scope.mark, batch).status == ScheduleStatus::invalid);
        value.done = true;
        return {};
    }
};

void failure_batches_resume_without_truncation_or_ticket_reuse() {
    for (const bool cleanup : {false, true}) {
        std::array<TaskSlot, 6> slots;
        Scheduler scheduler(slots, stack_bytes);
        FailureBatches value;
        const auto parent = scheduler.submit(cleanup ? CleanupChild::body : FailureBatches::run, &value,
                                             cleanup ? FailureBatches::run : nullptr);
        check(parent.status == ScheduleStatus::ok && scheduler.pump(40).status == ScheduleStatus::ok && value.done);
        check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
    }
}

Panic zero_report_capacity(Task &task, void *) noexcept {
    const auto scope = task.mark();
    check(scope.status == ScheduleStatus::ok);
    Work success;
    Work failure;
    failure.panic = {6, "needs space"};
    check(task.spawn(Work::run, &success, Work::clean).status == ScheduleStatus::ok);
    const auto child = task.spawn(Work::run, &failure, Work::clean);
    check(child.status == ScheduleStatus::ok);
    const auto full = task.close(scope.mark, std::span<ChildFailure>{});
    check(full.status == ScheduleStatus::report_full && full.reported == 0 && full.joined == 1 && full.panicked == 0);
    check(full.pending == child.ticket && full.first_failure.kind == OutcomeKind::pending);
    std::array<ChildFailure, 2> failures;
    failures[1].ticket.id = 777;
    const auto closed = task.close(scope.mark, failures);
    check(closed.status == ScheduleStatus::ok && closed.reported == 1 && closed.joined == 2 && closed.panicked == 1);
    check(failures[0].ticket == child.ticket && failures[0].outcome.panic.message() == "needs space");
    check(failures[1].ticket.id == 777 && success.cleanups == 1 && failure.cleanups == 1);
    return {};
}

void zero_capacity_reports_drain_successes_and_preserve_failed_child() {
    std::array<TaskSlot, 3> slots;
    Scheduler scheduler(slots, stack_bytes);
    const auto parent = scheduler.submit(zero_report_capacity, nullptr);
    check(parent.status == ScheduleStatus::ok && scheduler.pump(20).status == ScheduleStatus::ok);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
}

struct ReportRetry final {
public:
    std::array<ChildFailure, 2> failures;
    ScopeClose first;
    bool paused = false;
    bool done = false;

    static Panic first_child(Task &, void *) noexcept {
        char text[] = "first consumed";
        const Panic panic{6, {text, sizeof(text) - 1}};
        std::fill_n(text, sizeof(text), 'x');
        return panic;
    }

    static Panic second_child(Task &task, void *) noexcept {
        check(task.yield() == ContextStatus::ok);
        fail_release = true;
        char text[] = "second retained";
        const Panic panic{2, {text, sizeof(text) - 1}};
        std::fill_n(text, sizeof(text), 'y');
        return panic;
    }

    static Panic parent(Task &task, void *data) noexcept {
        auto &value = *static_cast<ReportRetry *>(data);
        const auto scope = task.mark();
        check(scope.status == ScheduleStatus::ok);
        const auto first = task.spawn(first_child, nullptr);
        const auto second = task.spawn(second_child, nullptr);
        check(first.status == ScheduleStatus::ok && second.status == ScheduleStatus::ok);
        value.failures[1].ticket.id = 777;
        value.first = task.close(scope.mark, value.failures);
        fail_release = false;
        check(value.first.status == ScheduleStatus::context_failed && value.first.reported == 1);
        check(value.first.joined == 1 && value.first.panicked == 1 && value.first.pending == second.ticket);
        check(value.first.pending_result.outcome.panic.message() == "second retained");
        check(value.failures[0].ticket == first.ticket && value.failures[1].ticket.id == 777);
        value.paused = true;
        check(task.yield() == ContextStatus::ok);
        const auto closed = task.close(scope.mark, value.failures);
        check(closed.status == ScheduleStatus::ok && closed.reported == 1 && closed.joined == 2 && closed.panicked == 2);
        check(closed.first_failure.panic.message() == "first consumed");
        check(value.failures[0].ticket == second.ticket && value.failures[1].ticket.id == 777);
        value.done = true;
        return {};
    }
};

void release_failure_does_not_publish_or_count_an_unconsumed_failure() {
    std::array<TaskSlot, 3> slots;
    Scheduler scheduler(slots, stack_bytes);
    ReportRetry value;
    const auto parent = scheduler.submit(ReportRetry::parent, &value);
    check(parent.status == ScheduleStatus::ok && scheduler.pump(6).resumed == 6);
    check(value.paused && !value.done && scheduler.inspect(parent.ticket).scopes == 1);
    check(scheduler.inspect(value.first.pending).state == TaskState::settled);
    check(scheduler.pump(2).status == ScheduleStatus::ok && value.done);
    check(scheduler.join(parent.ticket).status == ScheduleStatus::ok);
}

Panic leave_scope(Task &task, void *) noexcept {
    check(task.mark().status == ScheduleStatus::ok);
    return {};
}

void os_admission_failure() {
    std::array<TaskSlot, 1> slots;
    Scheduler scheduler(slots, stack_bytes);
    Work work;
    rlimit previous{};
    check(getrlimit(RLIMIT_AS, &previous) == 0);
    const rlimit limited{1, previous.rlim_max};
    check(setrlimit(RLIMIT_AS, &limited) == 0);
    const auto submitted = scheduler.submit(Work::run, &work, Work::clean);
    check(setrlimit(RLIMIT_AS, &previous) == 0);
    check(submitted.status == ScheduleStatus::context_failed && submitted.context.memory.error == ENOMEM);
    check(scheduler.inspect(submitted.ticket).state == TaskState::settled);
    const auto joined = scheduler.join(submitted.ticket);
    check(joined.status == ScheduleStatus::ok && joined.outcome.kind == OutcomeKind::spawn_failed);
    check(work.steps == 0 && work.cleanups == 0);
    std::printf("PASS scheduler kernel refusal: ENOMEM, no body, joined failure\n");
}

Panic fatal_body(Task &task, void *) noexcept {
    check(task.yield() == ContextStatus::ok);
    return {6, "body failed"};
}

Panic fatal_cleanup(Task &, void *) noexcept {
    constexpr std::string_view text = "release-trigger\n";
    check(::write(STDERR_FILENO, text.data(), text.size()) == static_cast<ssize_t>(text.size()));
    return {6, "release failed"};
}

struct Case final {
public:
    std::string_view name;
    void (*run)();
};

constexpr std::array cases{
    Case{"invalid_configuration_and_empty_queue", invalid_configuration_and_empty_queue},
    Case{"round_robin_and_bounded_pump", round_robin_and_bounded_pump},
    Case{"selector_handles_empty_settled_and_wrapped_cursors", selector_handles_empty_settled_and_wrapped_cursors},
    Case{"settled_tasks_hold_capacity_until_join", settled_tasks_hold_capacity_until_join},
    Case{"panic_settles_only_after_cleanup", panic_settles_only_after_cleanup},
    Case{"tickets_reject_other_schedulers_and_invalid_identity", tickets_reject_other_schedulers_and_invalid_identity},
    Case{"worker_affinity_rejects_foreign_access", worker_affinity_rejects_foreign_access},
    Case{"task_callbacks_cannot_reenter_scheduler", task_callbacks_cannot_reenter_scheduler},
    Case{"failed_admission_is_joinable_without_running_callbacks", failed_admission_is_joinable_without_running_callbacks},
    Case{"failed_admission_retains_rollback_mapping_until_join", failed_admission_retains_rollback_mapping_until_join},
    Case{"failed_join_preserves_settled_ticket", failed_join_preserves_settled_ticket},
    Case{"waiting_parent_keeps_locals_alive_and_releases_worker", waiting_parent_keeps_locals_alive_and_releases_worker},
    Case{"nested_children_use_fixed_slots_and_join_each_parent", nested_children_use_fixed_slots_and_join_each_parent},
    Case{"parent_cleanup_can_spawn_and_wait_for_its_child", parent_cleanup_can_spawn_and_wait_for_its_child},
    Case{"child_admission_obeys_parent_capacity_and_failure_ownership", child_admission_obeys_parent_capacity_and_failure_ownership},
    Case{"failed_child_release_retains_parent_ownership_until_retry", failed_child_release_retains_parent_ownership_until_retry},
    Case{"child_apis_reject_parent_sibling_root_and_foreign_capabilities", child_apis_reject_parent_sibling_root_and_foreign_capabilities},
    Case{"only_the_selected_child_wakes_its_waiting_parent", only_the_selected_child_wakes_its_waiting_parent},
    Case{"closing_scope_waits_while_locals_live_and_preserves_older_children", closing_scope_waits_while_locals_live_and_preserves_older_children},
    Case{"scope_nesting_capacity_and_generation_are_checked", scope_nesting_capacity_and_generation_are_checked},
    Case{"scope_operations_require_active_owner_and_worker", scope_operations_require_active_owner_and_worker},
    Case{"scope_close_reports_child_failures_without_hiding_later_children", scope_close_reports_child_failures_without_hiding_later_children},
    Case{"failure_batches_resume_without_truncation_or_ticket_reuse", failure_batches_resume_without_truncation_or_ticket_reuse},
    Case{"zero_capacity_reports_drain_successes_and_preserve_failed_child", zero_capacity_reports_drain_successes_and_preserve_failed_child},
    Case{"release_failure_does_not_publish_or_count_an_unconsumed_failure", release_failure_does_not_publish_or_count_an_unconsumed_failure},
};

}

extern "C" void *__real_mmap(void *, std::size_t, int, int, int, off_t);
extern "C" int __real_mprotect(void *, std::size_t, int);
extern "C" int __real_munmap(void *, std::size_t);

extern "C" void *__wrap_mmap(void *address, std::size_t size, int protection,
                             int flags, int descriptor, off_t offset) {
    if (fail_map) {
        errno = ENOMEM;
        return MAP_FAILED;
    }
    return __real_mmap(address, size, protection, flags, descriptor, offset);
}

extern "C" int __wrap_mprotect(void *address, std::size_t size, int protection) {
    if (fail_protect) {
        errno = EACCES;
        return -1;
    }
    return __real_mprotect(address, size, protection);
}

extern "C" int __wrap_munmap(void *address, std::size_t size) {
    if (fail_release) {
        errno = EIO;
        return -1;
    }
    return __real_munmap(address, size);
}

int main(int argc, char **argv) {
    if (argc == 2 && (std::string_view(argv[1]) == "--unclosed-body" || std::string_view(argv[1]) == "--unclosed-cleanup")) {
        const rlimit limit{0, 0};
        check(setrlimit(RLIMIT_CORE, &limit) == 0);
        std::array<TaskSlot, 1> slots;
        Scheduler scheduler(slots, stack_bytes);
        const bool body = std::string_view(argv[1]) == "--unclosed-body";
        check(scheduler.submit(body ? leave_scope : CleanupChild::body, nullptr, body ? nullptr : leave_scope).status == ScheduleStatus::ok);
        static_cast<void>(scheduler.pump(1));
        return 3;
    }
    if (argc == 2 && (std::string_view(argv[1]) == "--unjoined-body" || std::string_view(argv[1]) == "--unjoined-cleanup")) {
        const rlimit limit{0, 0};
        check(setrlimit(RLIMIT_CORE, &limit) == 0);
        std::array<TaskSlot, 2> slots;
        Scheduler scheduler(slots, stack_bytes);
        const bool body = std::string_view(argv[1]) == "--unjoined-body";
        check(scheduler.submit(body ? leave_child : CleanupChild::body, nullptr, body ? nullptr : leave_child).status == ScheduleStatus::ok);
        static_cast<void>(scheduler.pump(10));
        return 3;
    }
    if (argc == 2 && std::string_view(argv[1]) == "--os-admission-failure") {
        os_admission_failure();
        return 0;
    }
    if (argc == 2 && std::string_view(argv[1]) == "--fatal-task-cleanup") {
        const rlimit limit{0, 0};
        check(setrlimit(RLIMIT_CORE, &limit) == 0);
        std::array<TaskSlot, 1> slots;
        Scheduler scheduler(slots, stack_bytes);
        check(scheduler.submit(fatal_body, nullptr, fatal_cleanup).status == ScheduleStatus::ok);
        static_cast<void>(scheduler.pump(2));
        return 3;
    }
    if (argc != 1) {
        return 2;
    }
    for (const auto &test : cases) {
        test.run();
        std::printf("PASS %.*s\n", static_cast<int>(test.name.size()), test.name.data());
    }
    std::printf("%zu scheduler cases passed\n", cases.size());
    return 0;
}
