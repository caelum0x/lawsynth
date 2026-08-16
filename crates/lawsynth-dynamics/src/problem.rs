use crate::{
    ContinuousProblem, ControlledProblem, DelayedProblem, DiscreteProblem, ImplicitProblem,
};

/// The supported data-problem forms used by system-identification workflows.
#[derive(Clone, Debug, PartialEq)]
pub enum IdentificationProblem {
    Continuous(ContinuousProblem),
    Discrete(DiscreteProblem),
    Controlled(ControlledProblem),
    Delayed(DelayedProblem),
    Implicit(ImplicitProblem),
}

impl IdentificationProblem {
    pub fn observations(&self) -> usize {
        match self {
            Self::Continuous(problem) => problem.dataset().time().len(),
            Self::Discrete(problem) => problem.dataset().time().len(),
            Self::Controlled(problem) => problem.continuous().dataset().time().len(),
            Self::Delayed(problem) => problem.samples().current.len(),
            Self::Implicit(problem) => problem.dataset().time().len(),
        }
    }
}
