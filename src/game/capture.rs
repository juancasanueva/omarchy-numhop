//! A development aid: with `NUMHOP_CAPTURE=<dir>`, the game plays a short scripted
//! session and saves screenshots of each screen, then quits. It lets a change be
//! checked visually without anyone at the keyboard. Off unless the variable is set.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::path::PathBuf;

use bevy::input::ButtonState;
use bevy::input::InputSystems;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKeyCode};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use super::Screen;
use super::level::{Body, KeyPickup, Player};
use super::question::Asking;

#[derive(Debug, Clone)]
enum Step {
    Wait(f32),
    Tap(KeyCode),
    Hold(KeyCode, f32),
    /// Press a key and keep it down until `Release`, so shots can happen mid-move.
    Press(KeyCode),
    Release(KeyCode),
    Shot(&'static str),
    /// Type text, one character per key event, as a keyboard would.
    Type(&'static str),
    /// Put the player right under the first crystal and jump into it.
    JumpIntoCrystal,
    /// Put the player on the gate key.
    TouchKey,
    /// Press the number of the right option.
    AnswerCorrectly,
    /// Drop the player into a gap.
    Fall,
    Quit,
}

#[derive(Resource)]
struct Script {
    dir: PathBuf,
    steps: Vec<Step>,
    index: usize,
    timer: f32,
    held: Option<KeyCode>,
    tapped: Option<KeyCode>,
}

pub fn plugin(app: &mut App) {
    let Some(dir) = std::env::var_os("NUMHOP_CAPTURE").map(PathBuf::from) else { return };
    let _ = std::fs::create_dir_all(&dir);
    let scenario = std::env::var("NUMHOP_CAPTURE_SCENARIO").unwrap_or_default();
    let steps = if scenario == "quit-to-menu" { quit_to_menu() } else { full_tour() };
    app.insert_resource(Script {
        dir,
        steps,
        index: 0,
        timer: 0.0,
        held: None,
        tapped: None,
    })
    .add_systems(PreUpdate, drive.after(InputSystems).run_if(not(in_state(Screen::Loading))));
}

/// Start a match, ask to quit, then choose the main menu instead.
fn quit_to_menu() -> Vec<Step> {
    use Step::*;
    vec![
        Wait(2.0),
        Tap(KeyCode::Space),
        Wait(1.0),
        Tap(KeyCode::Space),
        Wait(0.8),
        Tap(KeyCode::KeyQ),
        Wait(0.6),
        Shot("01-confirm-quit"),
        Tap(KeyCode::KeyM),
        Wait(1.2),
        Shot("02-after-m"),
        Wait(0.6),
        Quit,
    ]
}

/// Every screen, from the menu to the results.
fn full_tour() -> Vec<Step> {
    use Step::*;
    let mut steps = vec![
        Wait(2.5),
        Shot("01-menu"),
        Tap(KeyCode::ArrowRight),
        Wait(0.6),
        Shot("02-menu-two-players"),
        // Type a name for the first player, then go back to the first row.
        Tap(KeyCode::ArrowDown),
        Wait(0.2),
        Tap(KeyCode::Enter),
        Wait(0.2),
        Type("Lucía"),
        Wait(0.4),
        Shot("02a-typing-name"),
        Tap(KeyCode::Enter),
        Wait(0.2),
        Tap(KeyCode::ArrowUp),
        Wait(0.2),
        // Up twice wraps from the first row to Settings, just above Start.
        Tap(KeyCode::ArrowUp),
        Wait(0.2),
        Tap(KeyCode::ArrowUp),
        Wait(0.2),
        Tap(KeyCode::Enter),
        Wait(0.5),
        Shot("02b-settings"),
        // Switch the language and back, to check the menu redraws at once.
        Tap(KeyCode::ArrowRight),
        Wait(0.4),
        Shot("02c-settings-switched"),
        Tap(KeyCode::ArrowLeft),
        Wait(0.3),
        Tap(KeyCode::Escape),
        Wait(0.4),
        Tap(KeyCode::Space),
        Wait(1.5),
        Shot("03-banner"),
        Tap(KeyCode::Space),
        // Jump early, before the first crystal overhead, then walk on.
        Press(KeyCode::ArrowRight),
        Wait(0.2),
        Press(KeyCode::Space),
        Wait(0.2),
        Shot("04b-jumping"),
        Release(KeyCode::Space),
        Wait(0.6),
        Shot("04a-walking"),
        Release(KeyCode::ArrowRight),
        Wait(0.8),
        Shot("04-running"),
        JumpIntoCrystal,
        Wait(1.2),
        Shot("05-question"),
        Tap(KeyCode::Digit1),
        Wait(1.0),
        Shot("06-feedback"),
        Tap(KeyCode::Space),
        Wait(1.0),
        Shot("07-after"),
        Tap(KeyCode::F1),
        Wait(0.6),
        Shot("08-help"),
        Tap(KeyCode::F1),
        Wait(0.4),
        Tap(KeyCode::Space),
        Wait(0.5),
        TouchKey,
        Wait(1.0),
        Shot("09-gate-question"),
        AnswerCorrectly,
        Wait(1.0),
        Shot("10-gate-feedback"),
        Tap(KeyCode::Space),
        Wait(1.0),
        Shot("11-world-cleared"),
        Tap(KeyCode::Space),
        Hold(KeyCode::ArrowRight, 1.0),
        Wait(0.5),
        Shot("12-dusk"),
    ];
    // Five falls use up every remaining life, ending the match.
    for _ in 0..5 {
        steps.extend([Fall, Wait(0.6), Tap(KeyCode::Space), Wait(0.6)]);
    }
    steps.extend([
        Wait(0.6),
        Shot("13-results"),
        Wait(0.8),
        Quit,
    ]);
    steps
}

fn drive(
    time: Res<Time<Real>>,
    mut script: ResMut<Script>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut player: Query<(&mut Transform, &mut Body), With<Player>>,
    keys_q: Query<&Transform, (With<KeyPickup>, Without<Player>)>,
    asking: Option<Res<Asking>>,
    mut typed: MessageWriter<KeyboardInput>,
    window: Query<Entity, With<PrimaryWindow>>,
    mut exit: MessageWriter<AppExit>,
) {
    // A tap lasts exactly one frame.
    if let Some(k) = script.tapped.take() {
        keys.release(k);
    }
    let Some(step) = script.steps.get(script.index).cloned() else { return };
    script.timer += time.delta_secs();
    let mut advance = true;
    match step {
        Step::Wait(s) => advance = script.timer >= s,
        Step::Tap(k) => {
            keys.press(k);
            script.tapped = Some(k);
        }
        Step::Hold(k, s) => {
            if script.held.is_none() {
                keys.press(k);
                script.held = Some(k);
            }
            advance = script.timer >= s;
            if advance {
                keys.release(k);
                script.held = None;
            }
        }
        Step::Type(text) => {
            if let Ok(window) = window.single() {
                for c in text.chars() {
                    typed.write(KeyboardInput {
                        key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified),
                        logical_key: Key::Character(c.to_string().into()),
                        state: ButtonState::Pressed,
                        text: Some(c.to_string().into()),
                        repeat: false,
                        window,
                    });
                }
            }
        }
        Step::Press(k) => keys.press(k),
        Step::Release(k) => keys.release(k),
        Step::Shot(name) => {
            let path = script.dir.join(format!("{name}.png"));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        }
        Step::JumpIntoCrystal => {
            if let Ok((mut tf, mut body)) = player.single_mut() {
                tf.translation = Vec3::new(8.5, 2.0, 0.0);
                body.vel = Vec2::new(0.0, 14.0);
            }
        }
        Step::TouchKey => {
            if let (Ok((mut tf, mut body)), Ok(key)) = (player.single_mut(), keys_q.single()) {
                tf.translation = key.translation - Vec3::Y * 0.4;
                body.vel = Vec2::ZERO;
            }
        }
        Step::AnswerCorrectly => {
            if let Some(asking) = &asking {
                let k = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4][asking.question.correct];
                keys.press(k);
                script.tapped = Some(k);
            }
        }
        Step::Fall => {
            if let Ok((mut tf, _)) = player.single_mut() {
                tf.translation.y = -10.0;
            }
        }
        Step::Quit => {
            exit.write(AppExit::Success);
        }
    }
    if advance {
        script.index += 1;
        script.timer = 0.0;
    }
}
