use lawsynth_scheduler::SchedulerTransport;

fn main() {
    eprintln!(
        "lawsynth-scheduler exposes {}; broker and network serving are not linked",
        SchedulerTransport::LocalTyped.reason()
    );
}
