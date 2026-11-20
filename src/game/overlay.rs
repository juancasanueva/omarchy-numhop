//! The help overlay (`?`) and the quit confirmation (`Q`). Either one pauses the game,
//! including the question clock, so reading the help never costs a life.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;

use super::assets::GameAssets;
use super::theme::{AccentBorder, AccentText, Rounding};
use super::ui::{self, label};
use super::{Game, Screen};

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    #[default]
    None,
    Help,
    ConfirmQuit,
}

#[derive(Component)]
struct OverlayRoot;

pub fn plugin(app: &mut App) {
    app.init_resource::<Overlay>()
        .add_systems(Update, (toggle, redraw.run_if(resource_changed::<Overlay>)).chain().run_if(not(in_state(Screen::Loading))));
}

/// Gameplay systems run only while no overlay is open.
pub fn no_overlay(overlay: Res<Overlay>) -> bool {
    *overlay == Overlay::None
}

const SHORTCUTS: &[(&str, &str)] = &[
    ("Left / Right or A / D", "Move"),
    ("Space, Up or W", "Jump"),
    ("1 2 3 4", "Pick an answer"),
    ("Arrow keys + Enter", "Choose an answer"),
    ("Enter / Space", "Continue"),
    ("?", "Show or hide this help"),
    ("Q", "Quit"),
];

fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    screen: Res<State<Screen>>,
    game: Option<Res<Game>>,
    mut overlay: ResMut<Overlay>,
    mut time: ResMut<Time<Virtual>>,
    mut exit: MessageWriter<AppExit>,
    mut commands: Commands,
    mut next: ResMut<NextState<Screen>>,
) {
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let help_pressed = keys.just_pressed(KeyCode::F1) || (keys.just_pressed(KeyCode::Slash) && shift);
    let mut wanted = *overlay;
    match *overlay {
        Overlay::None => {
            if help_pressed {
                wanted = Overlay::Help;
            } else if keys.just_pressed(KeyCode::KeyQ) {
                // Nothing to lose on the menu or the results screen, so quit at once.
                let in_game = game.is_some() && !matches!(screen.get(), Screen::Menu | Screen::Results);
                if in_game {
                    wanted = Overlay::ConfirmQuit;
                } else {
                    exit.write(AppExit::Success);
                }
            }
        }
        Overlay::Help => {
            if help_pressed || keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Enter) {
                wanted = Overlay::None;
            }
        }
        Overlay::ConfirmQuit => {
            if keys.just_pressed(KeyCode::KeyY) || keys.just_pressed(KeyCode::KeyQ) {
                exit.write(AppExit::Success);
            } else if keys.just_pressed(KeyCode::KeyM) {
                commands.remove_resource::<Game>();
                next.set(Screen::Menu);
                wanted = Overlay::None;
            } else if keys.just_pressed(KeyCode::KeyN) || keys.just_pressed(KeyCode::Escape) {
                wanted = Overlay::None;
            }
        }
    }
    if wanted != *overlay {
        *overlay = wanted;
        if wanted == Overlay::None {
            time.unpause();
        } else {
            time.pause();
        }
    }
}

fn redraw(
    mut commands: Commands,
    overlay: Res<Overlay>,
    roots: Query<Entity, With<OverlayRoot>>,
    assets: Res<GameAssets>,
    rounding: Res<Rounding>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    let a = &*assets;
    match *overlay {
        Overlay::None => {}
        Overlay::Help => {
            let rows: Vec<Entity> = SHORTCUTS
                .iter()
                .map(|(keys, what)| {
                    commands
                        .spawn((
                            Node {
                                width: percent(100),
                                justify_content: JustifyContent::SpaceBetween,
                                column_gap: px(40),
                                ..default()
                            },
                            children![
                                (label(a, *keys, 22.0, ui::TEXT), AccentText),
                                label(a, *what, 22.0, ui::MUTED)
                            ],
                        ))
                        .id()
                })
                .collect();
            let card = commands
                .spawn((ui::card(rounding.0), BackgroundColor(ui::PANEL_SOLID), BorderColor::all(ui::TEXT), AccentBorder))
                .with_child(label(a, "Shortcuts", 32.0, ui::TEXT))
                .add_children(&rows)
                .with_child(label(a, "Press ? or Esc to close", 18.0, ui::MUTED))
                .id();
            commands.spawn((OverlayRoot, ui::fullscreen(ui::DIM, 100))).add_child(card);
        }
        Overlay::ConfirmQuit => {
            let card = commands
                .spawn((ui::card(rounding.0), BackgroundColor(ui::PANEL_SOLID), BorderColor::all(ui::TEXT), AccentBorder))
                .with_children(|c| {
                    c.spawn(label(a, "Quit the game?", 34.0, ui::TEXT));
                    c.spawn(label(a, "The current match will be lost.", 20.0, ui::MUTED));
                    c.spawn((label(a, "Y  quit     M  main menu     N  keep playing", 22.0, ui::TEXT), AccentText));
                })
                .id();
            commands.spawn((OverlayRoot, ui::fullscreen(ui::DIM, 100))).add_child(card);
        }
    }
}
