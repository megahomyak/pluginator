use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use libloading::Library;

macro_rules! prepare_plugin_trait_macro {
    ($_:tt, $krate:ident) => {
        #[macro_export]
        macro_rules! plugin_trait {
            ($plugin_trait:path) => {
                #[macro_export]
                macro_rules! prepare_plugin_implementation_macro {
                    ($plugin_crate:path) => {
                        #[macro_export]
                        macro_rules! plugin_implementation {
                            ($initializer:expr) => {
                                #[no_mangle]
                                pub extern "C" fn get_interface() -> *mut dyn $plugin_crate::$plugin_trait {
                                    Box::into_raw(Box::new($initializer))
                                }
                            };
                        }
                    }
                }
                prepare_plugin_implementation_macro!($_ $krate);

                pub type LoadedPlugin = $crate::LoadedPlugin<dyn $plugin_trait>;

                pub fn load_plugin<Path: AsRef<std::path::Path>>(
                    path: Path,
                ) -> Result<$crate::LoadedPlugin<dyn $plugin_trait>, $crate::plugin::LoadingError> {
                    $crate::plugin::load(path)
                }
            };
        }
    };
}
prepare_plugin_trait_macro!($, crate);

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

    pub fn load<Path: AsRef<std::path::Path>, Plugin: ?Sized>(
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
