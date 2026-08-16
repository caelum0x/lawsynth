use std::collections::BTreeMap;

mod config;
mod convert;
mod error;
mod py_bundle;
mod py_dataset;
mod py_events;
mod py_plan;
mod py_run;
mod py_simulation;
mod py_world;

use lawsynth_bundle::{read_world, write_world};
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_differentiate::DerivativeMethod;
use lawsynth_discovery::{DiscoveryConfig, SparseMethod, discover};
use lawsynth_expr::parse;
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole, World};
use pyo3::{exceptions::PyValueError, prelude::*};

pub use config::PythonConfig;
pub use convert::identifier_values;
pub use error::message as error_message;
pub use py_bundle::{load_continuous_world, save_continuous_world};
pub use py_dataset::dataset_from_columns;
pub use py_events::scheduled_values;
pub use py_plan::state_identifiers;
pub use py_run::request_from_values;
pub use py_simulation::trajectory_values;
pub use py_world::equation_strings;

fn value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

#[pyclass(name = "Trajectory", skip_from_py_object)]
#[derive(Clone)]
struct PyTrajectory {
    #[pyo3(get)]
    time: Vec<f64>,
    #[pyo3(get)]
    values: BTreeMap<String, Vec<f64>>,
}

#[pyclass(name = "World")]
struct PyWorld {
    world: World,
}

#[pyclass(name = "Scenario")]
struct PyScenario {
    world: Py<PyWorld>,
    initial: BTreeMap<String, f64>,
    parameters: BTreeMap<String, f64>,
    inputs: BTreeMap<String, f64>,
    parameter_schedule: Vec<(f64, String, f64)>,
    input_schedule: Vec<(f64, String, f64)>,
}

#[pymethods]
impl PyWorld {
    #[new]
    #[pyo3(signature = (states, parameters, equations, controls=None))]
    fn new(
        states: Vec<String>,
        parameters: BTreeMap<String, f64>,
        equations: BTreeMap<String, String>,
        controls: Option<Vec<String>>,
    ) -> PyResult<Self> {
        let mut variables = states
            .into_iter()
            .map(|name| {
                Identifier::new(name)
                    .map(|id| Variable::new(id, VariableRole::State))
                    .map_err(value_error)
            })
            .collect::<PyResult<Vec<_>>>()?;
        variables.extend(
            controls
                .unwrap_or_default()
                .into_iter()
                .map(|name| {
                    Identifier::new(name)
                        .map(|id| Variable::new(id, VariableRole::Control))
                        .map_err(value_error)
                })
                .collect::<PyResult<Vec<_>>>()?,
        );
        let parameters = parameters
            .into_iter()
            .map(|(name, value)| {
                Identifier::new(name)
                    .map(|id| Parameter::new(id, value))
                    .map_err(value_error)
            })
            .collect::<PyResult<Vec<_>>>()?;
        let laws = equations
            .into_iter()
            .map(|(target, expression)| {
                Ok(ContinuousLaw::new(
                    Identifier::new(target).map_err(value_error)?,
                    parse(&expression).map_err(value_error)?,
                ))
            })
            .collect::<PyResult<Vec<_>>>()?;
        Ok(Self {
            world: World::new(variables, parameters, laws).map_err(value_error)?,
        })
    }

    #[pyo3(signature = (initial, start=0.0, end=1.0, step=0.01, parameters=None, inputs=None))]
    fn simulate(
        &self,
        initial: BTreeMap<String, f64>,
        start: f64,
        end: f64,
        step: f64,
        parameters: Option<BTreeMap<String, f64>>,
        inputs: Option<BTreeMap<String, f64>>,
    ) -> PyResult<PyTrajectory> {
        let mut request = SimulationRequest::default();
        for (name, value) in initial {
            request = request.with_initial(Identifier::new(name).map_err(value_error)?, value);
        }
        for (name, value) in parameters.unwrap_or_default() {
            request =
                request.with_parameter_override(Identifier::new(name).map_err(value_error)?, value);
        }
        for (name, value) in inputs.unwrap_or_default() {
            request = request.with_input(Identifier::new(name).map_err(value_error)?, value);
        }
        let trajectory = simulate(
            &self.world,
            SimulationConfig::new(start, end, step).map_err(value_error)?,
            &request,
        )
        .map_err(value_error)?;
        Ok(PyTrajectory {
            time: trajectory.time,
            values: trajectory
                .values
                .into_iter()
                .map(|(id, values)| (id.to_string(), values))
                .collect(),
        })
    }

    fn equations(&self) -> BTreeMap<String, String> {
        self.world
            .laws()
            .iter()
            .map(|(id, law)| (id.to_string(), lawsynth_expr::print(&law.expression)))
            .collect()
    }

    fn save(&self, path: String) -> PyResult<()> {
        write_world(path, &self.world).map_err(value_error)
    }

    #[staticmethod]
    fn load(path: String) -> PyResult<Self> {
        Ok(Self {
            world: read_world(path).map_err(value_error)?,
        })
    }
}

#[pymethods]
impl PyScenario {
    #[new]
    #[pyo3(signature = (world, initial, parameters=None, inputs=None, parameter_schedule=None, input_schedule=None))]
    fn new(
        world: Py<PyWorld>,
        initial: BTreeMap<String, f64>,
        parameters: Option<BTreeMap<String, f64>>,
        inputs: Option<BTreeMap<String, f64>>,
        parameter_schedule: Option<Vec<(f64, String, f64)>>,
        input_schedule: Option<Vec<(f64, String, f64)>>,
    ) -> Self {
        Self {
            world,
            initial,
            parameters: parameters.unwrap_or_default(),
            inputs: inputs.unwrap_or_default(),
            parameter_schedule: parameter_schedule.unwrap_or_default(),
            input_schedule: input_schedule.unwrap_or_default(),
        }
    }

    #[pyo3(signature = (start=0.0, end=1.0, step=0.01))]
    fn simulate(&self, py: Python<'_>, start: f64, end: f64, step: f64) -> PyResult<PyTrajectory> {
        let world = self.world.bind(py).borrow();
        let mut request = SimulationRequest::default();
        for (name, value) in &self.initial {
            request = request.with_initial(Identifier::new(name).map_err(value_error)?, *value);
        }
        for (name, value) in &self.parameters {
            request = request
                .with_parameter_override(Identifier::new(name).map_err(value_error)?, *value);
        }
        for (name, value) in &self.inputs {
            request = request.with_input(Identifier::new(name).map_err(value_error)?, *value);
        }
        for (time, name, value) in &self.parameter_schedule {
            request = request.with_scheduled_parameter(
                *time,
                Identifier::new(name).map_err(value_error)?,
                *value,
            );
        }
        for (time, name, value) in &self.input_schedule {
            request = request.with_scheduled_input(
                *time,
                Identifier::new(name).map_err(value_error)?,
                *value,
            );
        }
        let trajectory = simulate(
            &world.world,
            SimulationConfig::new(start, end, step).map_err(value_error)?,
            &request,
        )
        .map_err(value_error)?;
        Ok(PyTrajectory {
            time: trajectory.time,
            values: trajectory
                .values
                .into_iter()
                .map(|(id, values)| (id.to_string(), values))
                .collect(),
        })
    }
}

/// Discovers a continuous world from aligned numerical observations.
#[allow(clippy::too_many_arguments)] // PyO3 exposes these independently named Python keywords.
#[pyfunction]
#[pyo3(signature = (time, columns, states, polynomial_degree=2, threshold=0.05, solver="stlsq", include_trigonometric=false, include_rational=false, smoothing_radius=None, derivative_method="finite", savgol_window=5, tvreg_lambda=0.1, tvreg_iterations=100, symbolic_depth=None))]
fn discover_world(
    time: Vec<f64>,
    columns: BTreeMap<String, Vec<f64>>,
    states: Vec<String>,
    polynomial_degree: usize,
    threshold: f64,
    solver: &str,
    include_trigonometric: bool,
    include_rational: bool,
    smoothing_radius: Option<usize>,
    derivative_method: &str,
    savgol_window: usize,
    tvreg_lambda: f64,
    tvreg_iterations: usize,
    symbolic_depth: Option<usize>,
) -> PyResult<PyWorld> {
    let states = states
        .into_iter()
        .map(Identifier::new)
        .collect::<Result<Vec<_>, _>>()
        .map_err(value_error)?;
    let columns = columns
        .into_iter()
        .map(|(name, values)| {
            Identifier::new(name)
                .map(|id| NumericColumn::new(id, values))
                .map_err(value_error)
        })
        .collect::<PyResult<Vec<_>>>()?;
    let dataset =
        Dataset::new(TimeAxis::new(time).map_err(value_error)?, columns).map_err(value_error)?;
    let mut config = DiscoveryConfig::new(states);
    config.polynomial_degree = polynomial_degree;
    config.sparse.threshold = threshold;
    config.sparse_method = match solver {
        "stlsq" => SparseMethod::Stlsq,
        "sr3" => SparseMethod::Sr3,
        _ => return Err(value_error("solver must be 'stlsq' or 'sr3'")),
    };
    config.include_trigonometric = include_trigonometric;
    config.include_rational = include_rational;
    config.smoothing_radius = smoothing_radius;
    config.derivative.method = match derivative_method {
        "finite" => DerivativeMethod::FiniteDifference,
        "savgol" => DerivativeMethod::SavitzkyGolay {
            window: savgol_window,
        },
        "spline" => DerivativeMethod::NaturalCubicSpline,
        "spectral" => DerivativeMethod::Spectral,
        "tvreg" => DerivativeMethod::TotalVariation {
            lambda: tvreg_lambda,
            iterations: tvreg_iterations,
        },
        _ => {
            return Err(value_error(
                "derivative_method must be 'finite', 'savgol', 'spline', 'spectral', or 'tvreg'",
            ));
        }
    };
    config.symbolic = symbolic_depth.map(|max_depth| lawsynth_symbolic::SymbolicConfig {
        max_depth,
        ..Default::default()
    });
    let result = discover(&dataset, &config).map_err(value_error)?;
    let candidate = result
        .candidates
        .into_iter()
        .next()
        .ok_or_else(|| value_error("discovery produced no candidates"))?;
    Ok(PyWorld {
        world: candidate.world,
    })
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyWorld>()?;
    module.add_class::<PyScenario>()?;
    module.add_class::<PyTrajectory>()?;
    module.add_function(wrap_pyfunction!(discover_world, module)?)?;
    Ok(())
}
