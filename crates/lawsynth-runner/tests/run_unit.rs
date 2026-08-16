use lawsynth_runner::{
    CancellationToken, ResourceLimiter, ResourceRequest, WorkEnvelope, WorkProcess,
    classify_result, execute,
};

struct Sum;
impl WorkProcess for Sum {
    type Output = u64;
    fn execute(
        &mut self,
        envelope: &WorkEnvelope,
        cancellation: &CancellationToken,
    ) -> Result<u64, lawsynth_runner::RunnerError> {
        cancellation.check()?;
        Ok(envelope.input.iter().map(|byte| u64::from(*byte)).sum())
    }
}

#[test]
fn execute_releases_admitted_resources() {
    let resources = ResourceRequest::new(100, 1024, 0).unwrap();
    let envelope =
        WorkEnvelope::new("sum", "sum-bytes", 1, 1, 2, resources, vec![1, 2, 3]).unwrap();
    let mut limiter = ResourceLimiter::new(resources);
    let mut process = Sum;
    let token = CancellationToken::default();
    let result = execute(&mut limiter, &envelope, &mut process, &token);
    assert_eq!(result.unwrap(), 6);
    assert_eq!(limiter.reserved().memory_bytes, 0);
    assert_eq!(
        classify_result("sum", &Ok::<_, lawsynth_runner::RunnerError>(6)).status,
        lawsynth_runner::ExecutionStatus::Succeeded
    );
}
