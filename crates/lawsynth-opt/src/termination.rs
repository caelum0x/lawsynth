/// Why coordinate optimization stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminationReason {
    MinimumStep,
    IterationLimit,
}
