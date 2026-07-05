//! In-process Astro site management: manifest/scaffold (`sites`) and dev-server
//! preview lifecycle (`preview`). Absorbed from the former `astro-supervisor`
//! process — these run as direct calls inside the backend now, no IPC.
pub mod build;
pub mod error;
pub mod preview;
pub mod sites;
