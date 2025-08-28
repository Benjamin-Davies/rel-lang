#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "sync"))]
use alloc::rc::{Rc, Weak};
#[cfg(not(feature = "sync"))]
use core::cell::RefCell as Lock;
#[cfg(feature = "sync")]
use std::sync::{Arc as Rc, RwLock as Lock, Weak};

mod eval;
mod factories;
mod manager;
mod node;
mod ops;
mod shift;

pub use crate::{manager::Manager, node::Node};
