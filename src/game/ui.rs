//! Small UI building blocks shared by the screens.

use bevy::prelude::*;

use super::assets::GameAssets;

pub const PANEL: Color = Color::srgba(0.05, 0.05, 0.08, 0.86);
pub const PANEL_SOLID: Color = Color::srgb(0.07, 0.07, 0.10);
pub const DIM: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
pub const TEXT: Color = Color::srgb(0.97, 0.97, 0.98);
pub const MUTED: Color = Color::srgb(0.68, 0.70, 0.74);
pub const GOOD: Color = Color::srgb(0.35, 0.85, 0.45);
pub const BAD: Color = Color::srgb(0.95, 0.35, 0.35);
pub const TENS: Color = Color::srgb(0.95, 0.38, 0.35);
pub const UNITS: Color = Color::srgb(0.38, 0.62, 1.0);

pub fn font(assets: &GameAssets, size: f32) -> TextFont {
    TextFont {
        font: assets.font.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    }
}

pub fn symbol_font(assets: &GameAssets, size: f32) -> TextFont {
    TextFont {
        font: assets.symbols.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    }
}

pub fn label(assets: &GameAssets, text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (Text::new(text), font(assets, size), TextColor(color))
}

/// A full-window layer that centers its children, drawn above the game.
pub fn fullscreen(background: Color, z: i32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(14),
            ..default()
        },
        BackgroundColor(background),
        GlobalZIndex(z),
    )
}

pub fn card(radius: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        padding: UiRect::axes(px(36), px(28)),
        row_gap: px(14),
        border: UiRect::all(px(3)),
        border_radius: BorderRadius::all(px(radius)),
        min_width: px(520),
        ..default()
    }
}
