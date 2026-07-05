//! WebSocket protocol: connection plumbing (`socket`) and command → FSM →
//! side-effect orchestration (`dispatch`).
pub mod dispatch;
pub mod socket;
