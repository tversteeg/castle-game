use anyhow::{Context, Error};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    prelude::{AssetServer, Commands, Handle, Mesh, Res},
};
use bevy_inspector_egui::Inspectable;
use usvg::{Options, Tree};

#[derive(Debug, Clone, Inspectable)]
pub struct AlliedCharacterSvg(#[inspectable(ignore)] pub Handle<Mesh>);

/// Bevy SVG asset loader.
#[derive(Debug, Default)]
pub struct SvgAssetLoader;

impl AssetLoader for SvgAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            bevy::log::debug!("Loading SVG {:?}", load_context.path());

            // Parse and simplify the SVG file
            let svg_tree = Tree::from_data(&bytes, &Options::default().to_ref())
                .with_context(|| format!("Could not parse SVG file {:?}", load_context.path()))?;

            // Generate the mesh
            let mesh = crate::draw::mesh::svg_to_mesh(&svg_tree);

            // Upload the mesh
            load_context.set_default_asset(LoadedAsset::new(mesh));

            bevy::log::trace!("SVG loaded");

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg"]
    }
}

/// Load the SVG's as resources.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(AlliedCharacterSvg(
        asset_server.load("units/allies/character.svg"),
    ));
}
