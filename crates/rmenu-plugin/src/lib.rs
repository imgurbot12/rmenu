#[cfg(feature = "rmenu_internals")]
pub mod internal;

#[cfg(feature = "cache")]
pub mod cache;

use abi_stable::std_types::*;
use abi_stable::{sabi_trait, StableAbi};

#[cfg(feature = "cache")]
use serde::{Deserialize, Serialize};

/// Configured Entry Execution settings
#[repr(C)]
#[derive(Debug, Clone, StableAbi)]
#[cfg_attr(feature = "cache", derive(Serialize, Deserialize))]
pub enum Exec {
    Command(RString),
    Terminal(RString),
}

/// Module search-entry Icon configuration
#[repr(C)]
#[derive(Debug, Clone, StableAbi)]
#[cfg_attr(feature = "cache", derive(Serialize, Deserialize))]
pub struct Icon {
    pub name: RString,
    pub data: RVec<u8>,
}

/// Module single search-entry
#[repr(C)]
#[derive(Debug, Clone, StableAbi)]
#[cfg_attr(feature = "cache", derive(Serialize, Deserialize))]
pub struct Entry {
    pub name: RString,
    pub exec: Exec,
    pub comment: ROption<RString>,
    pub icon: ROption<Icon>,
}

/// Alias for FFI-safe vector of `Entry` object
pub type Entries = RVec<Entry>;

/// Alias for FFI-Safe hashmap for config entries
pub type ModuleConfig = RHashMap<RString, RString>;

/// Module trait abstraction for all search plugins
#[sabi_trait]
pub trait Module {
    extern "C" fn name(&self) -> RString;
    extern "C" fn prefix(&self) -> RString;
    extern "C" fn search(&mut self, search: RString) -> Entries;
}

/// Generates the required rmenu plugin resources
#[macro_export]
macro_rules! export_plugin {
    ($module:ident) => {
        #[no_mangle]
        extern "C" fn load_module(config: &ModuleConfig) -> Box<dyn Module> {
            let inst = $module::new(config);
            Box::new(inst)
        }
    };
}
