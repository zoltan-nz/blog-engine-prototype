//! Client protocol: transport-neutral connection (`connection`), the `/ws`
//! socket (`socket`) and command → FSM →
//! side-effect orchestration (`dispatch`).
pub mod connection;
pub mod dispatch;
pub mod socket;
