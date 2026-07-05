use crate::types::SiteState;
use thiserror::Error;

/// Inputs to the site machine. Client commands and process outcomes both
/// arrive as these events; the machine decides legality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiteEvent {
    ScaffoldSucceeded,
    BuildRequested,
    BuildSucceeded,
    BuildFailed { reason: String },
    DeleteRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid site transition from {state:?} on {event:?}")]
pub struct InvalidTransition {
    pub state: SiteState,
    pub event: SiteEvent,
}

/// Pure transition function. Entry into `Creating` and exit from `Deleting`
/// are map-level operations (insert/remove) handled by the dispatcher, not
/// transitions of this machine.
pub fn transition(state: SiteState, event: SiteEvent) -> Result<SiteState, InvalidTransition> {
    match (&state, &event) {
        (SiteState::Creating, SiteEvent::ScaffoldSucceeded) => Ok(SiteState::Ready),
        (SiteState::Ready | SiteState::BuildFailed { .. }, SiteEvent::BuildRequested) => {
            Ok(SiteState::Building)
        }
        (SiteState::Building, SiteEvent::BuildSucceeded) => Ok(SiteState::Ready),
        (SiteState::Building, SiteEvent::BuildFailed { reason }) => Ok(SiteState::BuildFailed {
            reason: reason.clone(),
        }),
        (SiteState::Ready | SiteState::BuildFailed { .. }, SiteEvent::DeleteRequested) => {
            Ok(SiteState::Deleting)
        }
        _ => Err(InvalidTransition { state, event }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creating_scaffold_succeeded_becomes_ready() {
        assert_eq!(
            transition(SiteState::Creating, SiteEvent::ScaffoldSucceeded),
            Ok(SiteState::Ready)
        );
    }

    #[test]
    fn ready_build_requested_becomes_building() {
        assert_eq!(
            transition(SiteState::Ready, SiteEvent::BuildRequested),
            Ok(SiteState::Building)
        );
    }

    #[test]
    fn build_failed_build_requested_retries_to_building() {
        assert_eq!(
            transition(
                SiteState::BuildFailed { reason: "x".into() },
                SiteEvent::BuildRequested
            ),
            Ok(SiteState::Building)
        );
    }

    #[test]
    fn building_build_succeeded_becomes_ready() {
        assert_eq!(
            transition(SiteState::Building, SiteEvent::BuildSucceeded),
            Ok(SiteState::Ready)
        );
    }

    #[test]
    fn building_build_failed_carries_reason() {
        assert_eq!(
            transition(
                SiteState::Building,
                SiteEvent::BuildFailed {
                    reason: "tsc".into()
                }
            ),
            Ok(SiteState::BuildFailed {
                reason: "tsc".into()
            })
        );
    }

    #[test]
    fn ready_delete_requested_becomes_deleting() {
        assert_eq!(
            transition(SiteState::Ready, SiteEvent::DeleteRequested),
            Ok(SiteState::Deleting)
        );
    }

    #[test]
    fn build_failed_delete_requested_becomes_deleting() {
        assert_eq!(
            transition(
                SiteState::BuildFailed { reason: "x".into() },
                SiteEvent::DeleteRequested
            ),
            Ok(SiteState::Deleting)
        );
    }

    #[test]
    fn building_delete_requested_is_rejected() {
        let result = transition(SiteState::Building, SiteEvent::DeleteRequested);
        assert!(result.is_err());
    }

    #[test]
    fn creating_build_requested_is_rejected() {
        let result = transition(SiteState::Creating, SiteEvent::BuildRequested);
        assert!(result.is_err());
    }

    #[test]
    fn deleting_accepts_no_events() {
        let result = transition(SiteState::Deleting, SiteEvent::BuildRequested);
        assert!(result.is_err());
    }

    #[test]
    fn ready_scaffold_succeeded_is_rejected() {
        let result = transition(SiteState::Ready, SiteEvent::ScaffoldSucceeded);
        assert_eq!(
            result,
            Err(InvalidTransition {
                state: SiteState::Ready,
                event: SiteEvent::ScaffoldSucceeded,
            })
        );
    }
}
