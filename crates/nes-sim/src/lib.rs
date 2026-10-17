//! 6502 + NES test machine.
//!
//! `cpu` is an NMOS 6502 core (official opcodes, binary-only ADC/SBC like the 2A03);
//! `nes` wraps it in a minimal mapper-0 NES used by end-to-end compiler tests.

pub mod cpu;
pub mod nes;
pub mod render;

pub use cpu::{Bus, Cpu, CpuError};
pub use nes::{NesBus, TestNes};
