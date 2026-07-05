//! Finite state machines governing domain state. Pure transition functions —
//! no I/O, no locks — so every arm is unit-testable. Side effects live in
//! `ws::dispatch`, which applies these transitions under the state locks.
pub mod preview;
pub mod site;
