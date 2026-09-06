#include "meowy/scheduler.hpp"

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
    check(joined.outcome.panic.code == 6 && joined.outcome.panic.message == "body failed");
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
