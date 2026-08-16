use crate::identifier_values;
use lawsynth_sim::SimulationRequest;
use std::collections::BTreeMap;
/// Builds a typed simulation request from Python mapping values.
pub fn request_from_values(
    initial: BTreeMap<String, f64>,
    parameters: BTreeMap<String, f64>,
    inputs: BTreeMap<String, f64>,
) -> Result<SimulationRequest, String> {
    let mut request = SimulationRequest::default();
    for (id, value) in identifier_values(initial)? {
        request = request.with_initial(id, value);
    }
    for (id, value) in identifier_values(parameters)? {
        request = request.with_parameter_override(id, value);
    }
    for (id, value) in identifier_values(inputs)? {
        request = request.with_input(id, value);
    }
    Ok(request)
}
