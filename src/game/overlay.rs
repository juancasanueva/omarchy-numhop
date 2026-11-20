//! The help overlay (`?`) and the quit confirmation (`Q`). Either one pauses the game,
//! including the question clock, so reading the help never costs a life.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::assets::GameAssets;
use super::menu::MenuChoices;
use super::theme::{AccentBorder, AccentText, Rounding};
use super::ui::{self, label};
use super::{Game, Language, Screen};

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

/// True when a key that types `?` was just pressed, whatever the keyboard layout: it is
/// Shift+/ on a US keyboard but Shift and the key right of 0 on a Spanish one.
fn question_mark_typed(typed: &mut MessageReader<KeyboardInput>) -> bool {
    typed
        .read()
        .any(|k| k.state.is_pressed() && matches!(&k.logical_key, Key::Character(c) if c.as_str() == "?"))
}

fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    menu: Res<MenuChoices>,
    screen: Res<State<Screen>>,
    game: Option<Res<Game>>,
    mut overlay: ResMut<Overlay>,
    mut time: ResMut<Time<Virtual>>,
    mut exit: MessageWriter<AppExit>,
    mut next: ResMut<NextState<Screen>>,
) {
    // Read the typed keys every frame, so none are left over when typing ends.
    let question_mark = question_mark_typed(&mut typed);
    // Q and ? are letters while a name is being typed.
    if menu.typing() {
        return;
    }
    let help_pressed = keys.just_pressed(KeyCode::F1) || question_mark;
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
            if keys.any_just_pressed([KeyCode::KeyY, KeyCode::KeyS, KeyCode::KeyQ]) {
                exit.write(AppExit::Success);
            } else if keys.just_pressed(KeyCode::KeyM) {
                // Only switch screens here. Gameplay systems still running this frame
                // need the match; the menu drops it when it opens.
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
    language: Res<Language>,
) {
    let t = language.t();
    for e in &roots {
        commands.entity(e).despawn();
    }
    let a = &*assets;
    match *overlay {
        Overlay::None => {}
        Overlay::Help => {
            let rows: Vec<Entity> = t
                .shortcuts
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
                .with_child(label(a, t.shortcuts_title, 32.0, ui::TEXT))
                .add_children(&rows)
                .with_child(label(a, t.close_help, 18.0, ui::MUTED))
                .id();
            commands.spawn((OverlayRoot, ui::fullscreen(ui::DIM, 100))).add_child(card);
        }
        Overlay::ConfirmQuit => {
            let card = commands
                .spawn((ui::card(rounding.0), BackgroundColor(ui::PANEL_SOLID), BorderColor::all(ui::TEXT), AccentBorder))
                .with_children(|c| {
                    c.spawn(label(a, t.quit_title, 34.0, ui::TEXT));
                    c.spawn(label(a, t.quit_lost, 20.0, ui::MUTED));
                    c.spawn((label(a, t.quit_keys, 22.0, ui::TEXT), AccentText));
                })
                .id();
            commands.spawn((OverlayRoot, ui::fullscreen(ui::DIM, 100))).add_child(card);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::state::app::StatesPlugin;

    use super::*;

    /// A `?` typed with whatever physical key the layout puts it on.
    fn question_mark(key_code: KeyCode) -> KeyboardInput {
        KeyboardInput {
            key_code,
            logical_key: Key::Character("?".into()),
            state: ButtonState::Pressed,
            text: Some("?".into()),
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    fn overlay_app() -> App {
        let mut app = App::new();
        app.add_plugins(StatesPlugin)
            .init_state::<Screen>()
            .add_message::<KeyboardInput>()
            .add_message::<AppExit>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<MenuChoices>()
            .init_resource::<Overlay>()
            .init_resource::<Time<Virtual>>()
            .add_systems(Update, toggle);
        app
    }

    #[test]
    fn question_mark_opens_help_on_any_layout() {
        // US: Shift+/. Spanish: Shift and the key right of 0, which reports as Minus.
        for key in [KeyCode::Slash, KeyCode::Minus] {
            let mut app = overlay_app();
            app.world_mut().write_message(question_mark(key));
            app.update();
            assert_eq!(*app.world().resource::<Overlay>(), Overlay::Help, "{key:?}");
        }
    }

    #[test]
    fn a_question_mark_while_typing_a_name_does_not_open_help() {
        let mut app = overlay_app();
        app.world_mut().resource_mut::<MenuChoices>().start_typing_for_test();
        app.world_mut().write_message(question_mark(KeyCode::Minus));
        app.update();
        assert_eq!(*app.world().resource::<Overlay>(), Overlay::None);
    }
}
