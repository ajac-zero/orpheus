/// High-level lifecycle events emitted while an agent run is in progress.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    ToolStarted {
        turn: usize,
        call_id: String,
        name: String,
        arguments: String,
    },
    ToolFinished {
        turn: usize,
        call_id: String,
        name: String,
        output: String,
    },
    ToolFailed {
        turn: usize,
        call_id: String,
        name: String,
        error: String,
    },
    TurnCompleted {
        turn: usize,
        response_id: String,
        function_calls: usize,
    },
    Completed {
        turns: usize,
        response_id: String,
    },
}
