/*
 *  Internal Library Loading Implementation
 */
use abi_stable::std_types::{RBox, RHashMap, RString};
use libloading::{Error, Library, Symbol};

use super::{Module, ModuleConfig};

/* Types */

pub struct Plugin {
    pub lib: Library,
    pub module: Box<dyn Module>,
}

type LoadFunc = unsafe extern "C" fn(&ModuleConfig) -> Box<dyn Module>;

/* Functions */

pub unsafe fn load_plugin(path: &str, cfg: &ModuleConfig) -> Result<Plugin, Error> {
    // Load and initialize library
    #[cfg(target_os = "linux")]
    let lib: Library = {
        // Load library with `RTLD_NOW | RTLD_NODELETE` to fix a SIGSEGV
        // https://github.com/nagisa/rust_libloading/issues/41#issuecomment-448303856
        libloading::os::unix::Library::open(Some(path), 0x2 | 0x1000)?.into()
    };
    #[cfg(not(target_os = "linux"))]
    let lib = Library::new(path.as_ref())?;
    // load module object and generate plugin
    let module = lib.get::<Symbol<LoadFunc>>(b"load_module")?(cfg);
    Ok(Plugin { lib, module })
}
