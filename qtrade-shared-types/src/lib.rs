use serde::{Deserialize, Serialize};

/// ArbitrageResult represents the result of the solver's optimization process
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbitrageResult {
    /// Delta values (tender amounts) for each pool
    pub deltas: Vec<Vec<f64>>,
    /// Lambda values (receive amounts) for each pool
    pub lambdas: Vec<Vec<f64>>,
    /// A-matrix that maps global to local indices
    pub a_matrices: Vec<Vec<Vec<f64>>>,
    /// Status of the optimization problem
    pub status: String,
}
