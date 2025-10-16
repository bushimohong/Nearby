use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use dioxus_component::App;
mod dioxus_component;
mod core;

fn main() {
	let cfg = Config::new().with_window(
		WindowBuilder::new()
			.with_title("Nearby")
			.with_always_on_top(false)
			.with_resizable(true),
	);
	LaunchBuilder::new().with_cfg(cfg).launch(App);
}