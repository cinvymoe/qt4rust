// MVI Core Traits

/// Intent trait - represents user intentions
pub trait Intent: std::fmt::Debug + Clone {}

/// State trait - represents application state
pub trait State: std::fmt::Debug + Clone {}

/// Reducer trait - transforms state based on intent
pub trait Reducer<S: State, I: Intent> {
    fn reduce(&self, state: S, intent: I) -> S;
}
