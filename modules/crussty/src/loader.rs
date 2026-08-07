//! dlopen wrapper for the bundled Crussty CE native libraries
//! (libpaper_native_jni.so / libpaper_native_chunk_encode_jni.so).

use libloading::Library;
use std::ffi::c_void;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// Libraries must outlive the JVM: `register_natives` stashes raw function
/// pointers into method entries, so dlclose when the plugin's handle goes
/// out of scope would leave them dangling. Keep every dlopen'd handle alive
/// for the process lifetime (leaked on purpose).
static KEEP_ALIVE: OnceLock<Mutex<Vec<&'static Library>>> = OnceLock::new();

pub struct NativeLib {
    lib: &'static Library,
}

impl NativeLib {
    /// # Safety
    /// `path` must name a valid, position-independent shared object that is
    /// safe to dlopen into this process.
    pub unsafe fn new(path: &Path) -> Result<Self, String> {
        let lib = unsafe { Library::new(path) }.map_err(|e| e.to_string())?;
        let lib: &'static Library = Box::leak(Box::new(lib));
        KEEP_ALIVE
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap()
            .push(lib);
        Ok(Self { lib })
    }

    /// Resolve an exported symbol by exact name (a JNI `Java_...` symbol).
    pub fn symbol(&self, name: &str) -> Option<*mut c_void> {
        unsafe {
            self.lib
                .get::<*mut c_void>(name.as_bytes())
                .ok()
                .map(|s| *s)
        }
    }
}
