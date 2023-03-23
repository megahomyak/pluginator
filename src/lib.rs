use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use libloading::Library;

#[macro_export]
macro_rules! plugin_trait {
    ($plugin_trait:path) => {
        pub unsafe fn load_plugin<Path: AsRef<std::path::Path>>(
            path: Path,
        ) -> Result<$crate::LoadedPlugin<dyn $plugin_trait>, $crate::plugin::LoadingError> {
            unsafe { $crate::plugin::load(path) }
        }
    };
}

#[macro_export]
macro_rules! plugin_implementation {
    ($plugin_trait:path, $initializer:expr) => {
        #[no_mangle]
        pub extern "C" fn get_interface() -> *mut dyn $plugin_trait {
            Box::into_raw(Box::new($initializer))
        }
    };
}

pub struct LoadedPlugin<Plugin: ?Sized> {
    library: ManuallyDrop<Library>,
    plugin: ManuallyDrop<Box<Plugin>>,
}

impl<Plugin: ?Sized> Drop for LoadedPlugin<Plugin> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.plugin);
            ManuallyDrop::drop(&mut self.library);
        }
    }
}

impl<Plugin: ?Sized> Deref for LoadedPlugin<Plugin> {
    type Target = Plugin;

    fn deref(&self) -> &Self::Target {
        self.plugin.as_ref()
    }
}

impl<Plugin: ?Sized> DerefMut for LoadedPlugin<Plugin> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.plugin.as_mut()
    }
}

pub mod plugin {
    use std::mem::ManuallyDrop;

    use super::LoadedPlugin;

    use libloading::{Library, Symbol};

    #[derive(Debug)]
    pub enum LoadingError {
        OpeningError(libloading::Error),
        InterfaceGettingError(libloading::Error),
    }

    pub unsafe fn load<Path: AsRef<std::path::Path>, Plugin: ?Sized>(
        path: Path,
    ) -> Result<LoadedPlugin<Plugin>, LoadingError> {
        let library =
            unsafe { Library::new(path.as_ref()) }.map_err(|e| LoadingError::OpeningError(e))?;
        let get_interface: Symbol<fn() -> *mut Plugin> =
            unsafe { library.get(b"get_interface") }
                .map_err(|e| LoadingError::InterfaceGettingError(e))?;
        let plugin = unsafe { Box::from_raw(get_interface()) };
        Ok(LoadedPlugin {
            plugin: ManuallyDrop::new(plugin),
            library: ManuallyDrop::new(library),
        })
    }
}
