#[derive(Debug, PartialEq, Clone)]
pub enum CompilationResult {
    Success,
    Failure,
    Warning,
    Pending,
}