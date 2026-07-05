use crate::types::PreviewState;
use thiserror::Error;

/// Inputs to the preview machine. `StartRequested` from `Failed` is the retry
/// path; `Failed` can arrive from the readiness poll (`Starting`) or a crashed
/// dev server (`Running`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewEvent {
    StartRequested,
    ServerReady,
    StopRequested,
    ProcessStopped,
    Failed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid preview transition from {state:?} on {event:?}")]
pub struct InvalidTransition {
    pub state: PreviewState,
    pub event: PreviewEvent,
}

pub fn transition(
    state: PreviewState,
    event: PreviewEvent,
) -> Result<PreviewState, InvalidTransition> {
    match (&state, &event) {
        (PreviewState::Stopped | PreviewState::Failed { .. }, PreviewEvent::StartRequested) => {
            Ok(PreviewState::Starting)
        }
        (PreviewState::Starting, PreviewEvent::ServerReady) => Ok(PreviewState::Running),
        (PreviewState::Running, PreviewEvent::StopRequested) => Ok(PreviewState::Stopping),
        (PreviewState::Stopping, PreviewEvent::ProcessStopped) => Ok(PreviewState::Stopped),
        (PreviewState::Starting | PreviewState::Running, PreviewEvent::Failed { reason }) => {
            Ok(PreviewState::Failed {
                reason: reason.clone(),
            })
        }
        _ => Err(InvalidTransition { state, event }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopped_start_requested_becomes_starting() {
        assert_eq!(
            transition(PreviewState::Stopped, PreviewEvent::StartRequested),
            Ok(PreviewState::Starting)
        );
    }

    #[test]
    fn failed_start_requested_retries_to_starting() {
        assert_eq!(
            transition(
                PreviewState::Failed { reason: "x".into() },
                PreviewEvent::StartRequested
            ),
            Ok(PreviewState::Starting)
        );
    }

    #[test]
    fn starting_server_ready_becomes_running() {
        assert_eq!(
            transition(PreviewState::Starting, PreviewEvent::ServerReady),
            Ok(PreviewState::Running)
        );
    }

    #[test]
    fn running_stop_requested_becomes_stopping() {
        assert_eq!(
            transition(PreviewState::Running, PreviewEvent::StopRequested),
            Ok(PreviewState::Stopping)
        );
    }

    #[test]
    fn stopping_process_stopped_becomes_stopped() {
        assert_eq!(
            transition(PreviewState::Stopping, PreviewEvent::ProcessStopped),
            Ok(PreviewState::Stopped)
        );
    }

    #[test]
    fn starting_failed_carries_reason() {
        assert_eq!(
            transition(
                PreviewState::Starting,
                PreviewEvent::Failed {
                    reason: "timeout".into()
                }
            ),
            Ok(PreviewState::Failed {
                reason: "timeout".into()
            })
        );
    }

    #[test]
    fn running_failed_carries_reason() {
        assert_eq!(
            transition(
                PreviewState::Running,
                PreviewEvent::Failed {
                    reason: "crashed".into()
                }
            ),
            Ok(PreviewState::Failed {
                reason: "crashed".into()
            })
        );
    }

    #[test]
    fn starting_start_requested_is_rejected() {
        let result = transition(PreviewState::Starting, PreviewEvent::StartRequested);
        assert_eq!(
            result,
            Err(InvalidTransition {
                state: PreviewState::Starting,
                event: PreviewEvent::StartRequested,
            })
        );
    }

    #[test]
    fn running_start_requested_is_rejected() {
        let result = transition(PreviewState::Running, PreviewEvent::StartRequested);
        assert!(result.is_err());
    }

    #[test]
    fn stopped_stop_requested_is_rejected() {
        let result = transition(PreviewState::Stopped, PreviewEvent::StopRequested);
        assert!(result.is_err());
    }

    #[test]
    fn starting_stop_requested_is_rejected() {
        let result = transition(PreviewState::Starting, PreviewEvent::StopRequested);
        assert!(result.is_err());
    }
}
