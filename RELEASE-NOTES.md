# Plotters Release Notes 

This documents contains the release notes for every major release since v0.3.

## Plotters v0.3

Plotters v0.3 is shipped with multiple major improvements.

## Plug-and-play backend importing

### Introduction 

Previously, Plotters implements all supported backend in the core crate. As the project is becoming bigger and bigger and
more and more backend is supported, those backend implementation cause a huge mantainance burden. 

For example, when `cairo-rs` crate is updated, plotters should release a new version with updated `cairo-rs` dependency for 
our cairo backend. However, most of the users doesn't actually use this backend and pushing new version for updating cairo backend
seems to be annoying for most of the people. As more and more backend is added, the depdendency is out of control.

### Details

To address this, we are now move all backend implementation code out of the plotters crate. To use a specific backend, you need to
explicitly add backend crate to your dependency. For example, to use bitmap backend for v0.2.x, we can do this

```rust
use plotters::prelude::*;
fn main() {
	let backend = BitMapBackend::new(...)
}
```

After this update, you should do the following code instead:

```rust
use plotters::prelude::*;
use plotter_bitmap::BitMapBackend;

//...

```

### Backward compatibility

Plotters v0.3 now have a concept of tier 1 backends, which is the most supported. Currently we have two tier 1 backends: `plotters-bitmap` and 
`plotters-svg`. Tier 1 backends are used by most of the people. 

To ease the upgrade for tier 1 backends, we still keep feature options in the core crate that can opt in those crates. And this is enabled by default.

Thus, if plotters is imported with default feature set, there would require no change. If the default feature set is opt out, then the following change
should be make with your `Crates.toml`: 

```toml
plotters = {version = "0.3", default_features = false, features = ["bitmap_backend", "svg_backend"]} # Instead of using feature "bitmap" and "svg"
```

For non tier 1 backends, manmually import is required (Please note tier on backends can be imported in same way). For example:

```toml
plotters = {version = "0.3", default_features = false} # Instead of having features = ["cairo"] at this point
plotters-cairo = "0.3" # We should import the cairo backend in this way.
```

And in your code, instead of import `plotters::prelude::CairoBackend`, you should import `plotters_cairo::CairoBackend`
