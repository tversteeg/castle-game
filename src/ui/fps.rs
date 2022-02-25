use bevy::{
    core::Name,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::Rect,
    prelude::{AssetServer, Color, Commands, Component, Query, Res, TextBundle, With},
    text::{Text, TextSection, TextStyle},
    ui::{AlignSelf, PositionType, Style, Val},
};

/// The FPS UI component.
#[derive(Component)]
pub struct FpsText;

/// Show the text with the FPS.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/Pixerif.ttf"),
                            font_size: 12.0,
                            color: Color::ORANGE_RED,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/Pixerif.ttf"),
                            font_size: 12.0,
                            color: Color::GOLD,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FpsText)
        .insert(Name::new("FPS Text"));
}

/// The sytem for updating the FPS.
pub fn system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                // Update the value of the second section
                text.sections[1].value = format!("{:.2}", average);
            }
        }
    }
}
