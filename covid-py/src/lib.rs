use pyo3::{prelude::*, wrap_pyfunction};

extern crate pyo3;
use covid::prelude as rs;
use covid::prelude::{Age, AnySampler, Real};
use paste::paste;
use pyo3::types::PyDict;
use toml;

// use pyo3::prelude::*;
use pyo3::exceptions::*;
use pythonize::{depythonize, pythonize};

type GenericSimulation = rs::Simulation<AnySampler>;

macro_rules! py_mutable_props {
    ($name:ident { $($x:ident : $t:ident),* }) => {
        paste! {
            #[pymethods]
            impl $name {
                $(
                    #[getter]
                    pub fn [<get_ $x>](&self) -> PyResult<$t> {
                        Ok(self.data.$x())
                    }

                    #[setter]
                    pub fn [<set_ $x>](&mut self, value: $t) -> PyResult<()> {
                        self.data.[<set_ $x>](value);
                        Ok(())
                    }
                )*
            }
        }
    };
}

macro_rules! py_immutable_props {
    ($name:ident { $($x:ident : $t:ident),*}) => {
        paste! {
            #[pymethods]
            impl $name {
                $(
                    #[getter]
                    pub fn [<get_ $x>](&self) -> PyResult<$t> {
                        Ok(self.data.$x())
                    }
                )*
            }
        }
    };
}

#[pyclass]
#[derive(Debug)]
pub struct Agent {
    data: rs::Ag,
}

impl Agent {
    fn new_from_data(agent: &rs::Ag) -> Self {
        Agent {
            data: agent.clone(),
        }
    }
}

#[pymethods]
impl Agent {
    #[new]
    fn new(age: u8) -> Self {
        Agent {
            data: rs::Ag::new(age),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.data).to_string())
    }
}

py_mutable_props!(Agent { age: Age });
py_immutable_props!(Agent {
    is_susceptible: bool,
    is_infecting: bool,
    secondary_infections: usize
});

#[pyclass]
#[derive(Debug)]
pub struct Simulation {
    data: GenericSimulation,
}

impl Simulation {
    // fn with_report<F, T>(&self, f: F) -> T
    // where
    //     F: FnOnce(rs::Reporter<AnySampler>) -> T,
    // {
    //     let report = rs::Reporter::new(&self.data);
    //     f(report)
    // }
    fn report(&self) -> rs::Report<AnySampler> {
        rs::Report::new(&self.data)
    }
}

#[pymethods]
impl Simulation {
    #[new]
    #[args(params = "None", params_voc = "None")]
    fn new(
        agents: Vec<PyRef<Agent>>,
        params: Option<&PyDict>,
        params_voc: Option<&PyDict>,
    ) -> PyResult<Self> {
        let mut builder = rs::PopBuilder::new(0);
        for agent in agents {
            builder.push_agent(&agent.data);
        }
        let sampler: rs::AnySampler = rs::SimpleSampler::new(5.5, 0.1).into();
        let mut data = builder.build(sampler);
        if let Some(p) = params {
            data.set_params_baseline(depythonize(p.as_ref())?);
        }
        if let Some(p) = params_voc {
            data.set_params_voc(depythonize(p.as_ref())?);
        }
        return Ok(Simulation { data });
    }

    // fn __str__(&self) -> PyResult<String> {
    //     let data: String = toml::to_string(&self.report().results()).unwrap();
    //     Ok(data)
    // }

    #[args(steps = "1")]
    fn run(&mut self, steps: usize) {
        self.data.run(steps);
    }

    fn seed(&mut self, seed: u64) {
        self.data.seed(seed);
    }

    fn curve(&self, n: usize) -> Vec<Real> {
        self.data.curve(n)
    }

    fn agent(&self, idx: usize) -> PyResult<Agent> {
        self.data
            .agents()
            .get(idx)
            .map(|a| Agent::new_from_data(&a))
            .ok_or_else(|| PyIndexError::new_err(idx))
    }

    // fn results(&self) -> PyResult<PyObject> {
    //     let results = self.report().results();

    //     Python::with_gil(|py| {
    //         let dict = PyDict::new(py);
    //         dict.set_item("params", pythonize(py, &results.params)?)?;
    //         dict.set_item("stats", pythonize(py, &results.stats)?)?;
    //         Ok(dict.as_ref().into())
    //     })
    // }
}

py_immutable_props!(Simulation { n_iter: usize });

/// Builds a new population from scratch and return a list of agents.
#[pyfunction(n = "0", kwds = "**")]
fn build_population(n: usize, kwds: Option<&PyDict>) -> PyResult<Vec<Agent>> {
    let mut builder = rs::PopBuilder::new(n);
    let gil = Python::acquire_gil();
    let py = gil.python();

    if let Some(dic) = kwds {
        // Parameters
        if let Some(v) = dic.get_item("prob_voc") {
            builder.set_prob_voc(v.extract()?);
        }

        // Init list of agents
        if let Some(v) = dic.get_item("age_counts") {
            if n != 0 {
                return Err(PyTypeError::new_err(
                    "cannot set n an and age_counts at the same time",
                ));
            }
            builder.age_counts(v.extract()?);
        } else if let Some(v) = dic.get_item("age_distrib") {
            builder.age_distribution(v.extract()?);
        }

        if let Some(lst) = dic.get_item("agents") {
            let data: Vec<Py<Agent>> = lst.extract()?;
            builder.push_agents(data.iter().map(|a| (&a.borrow(py)).data));
        }

        // Init infection strategies
        if let Some(v) = dic.get_item("exposed") {
            builder.contaminate_at_random(v.extract()?, false.into());
        }
        if let Some(v) = dic.get_item("infected") {
            builder.contaminate_at_random(v.extract()?, true.into());
        }
    }

    Ok(builder
        .agents()
        .iter()
        .map(|a| Agent::new_from_data(a))
        .collect::<Vec<_>>())
}

#[pymodule]
fn epirust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Agent>()?;
    m.add_class::<Simulation>()?;
    m.add_function(wrap_pyfunction!(build_population, m)?)?;

    Ok(())
}
