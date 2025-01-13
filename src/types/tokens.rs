use serde::Deserialize;

#[pyo3::pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct TopLogProbs {
    token: String,
    logprob: i32,
    bytes: Option<Vec<u8>>,
}

#[pyo3::pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct Content {
    token: String,
    logprob: i32,
    bytes: Option<Vec<u8>>,
    top_logprobs: TopLogProbs,
}

#[pyo3::pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
struct Refusal {
    token: String,
    logprob: i32,
    bytes: Option<Vec<u8>>,
    top_logprobs: TopLogProbs,
}

#[pyo3::pyclass(get_all)]
#[derive(Debug, Clone, Deserialize)]
pub struct LogProbs {
    content: Option<Vec<Content>>,
    refusal: Option<Vec<Refusal>>,
}
