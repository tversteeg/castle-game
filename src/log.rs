use bevy::{
    log::LogSettings,
    prelude::{App, Plugin},
};
use tracing_subscriber::{fmt::Layer, prelude::*, registry::Registry, EnvFilter};

/// Define our own log plugin so we can configure tracing.
pub struct CustomLogPlugin;

impl Plugin for CustomLogPlugin {
    #[cfg(target_arch = "wasm32")]
    fn build(&self, app: &mut App) {
        // Add traces to the console output
        tracing_wasm::set_as_global_default();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn build(&self, app: &mut App) {
        let default_filter = {
            let settings = app.world.get_resource_or_insert_with(LogSettings::default);
            format!("{},{}", settings.level, settings.filter)
        };
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&default_filter))
            .unwrap();
        let subscriber = Registry::default().with(filter_layer);

        let subscriber = subscriber.with(Layer::default());

        bevy::utils::tracing::subscriber::set_global_default(subscriber)
			.expect("Could not set global default tracing subscriber. If you've already set up a tracing subscriber, please disable LogPlugin from Bevy's DefaultPlugins");
    }
}
