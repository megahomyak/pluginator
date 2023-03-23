# Usage

Use `plugin_trait!` and `plugin_implementation!` in different crates.

Wrap the plugin trait with the `plugin_trait!` macro. A `load_plugin` function will be generated. It can load a plugin object from a static library.

Wrap the plugin implementation with a `plugin_implementation!` macro. A `get_interface` function will be generated. Compile the crate as a static library to use the plugin.

# Safety

`load_plugin` is unsafe because you should pass the *correct* (matching) plugin trait as its first argument, otherwise undefined behavior will happen. This happens because it is impossible to determine the return value type of an interface getter in a static library.

# Example

* `app/src/main.rs`:

```
fn main() {
    let plugin = unsafe { app::load_plugin("plugins/libplugin1.so") }.unwrap();
    plugin.print("hello");
}
```

* `app/src/lib.rs`:

```
pub trait Plugin: Sync + Send {
    fn print(&self, message: &str);
}

pluginator::plugin_trait!(Plugin);
```

* `plugin1/src/lib.rs`:

```
struct Plugin;

impl app::Plugin for Plugin {
    fn print(&self, message: &str) {
        println!("{}", message);
    }
}

pluginator::plugin_implementation!(app::Plugin, Plugin);
```
