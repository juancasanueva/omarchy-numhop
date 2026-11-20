//! The math moment: a question with four options and a 30-second clock.
//!
//! A right answer keeps the controller; a wrong answer or running out of time costs a
//! life. Either way the worked solution is shown before play resumes.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button, observe};

use crate::domain::math::{self, Prompt, Question};
use crate::domain::rules::{GEMS_PER_CRYSTAL, QUESTION_SECONDS, SCORE_CRYSTAL};

use super::assets::GameAssets;
use super::banner::{after_death, after_gate};
use super::hud::Toast;
use super::level::{CrystalBlock, Grid, LevelEntity, cell_center};
use super::overlay::no_overlay;
use super::theme::{Accent, AccentBackground, AccentBorder, AccentText, Rounding};
use super::ui::{self, label, symbol_font};
use super::{Game, GameRng, Screen, Worlds};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskKind {
    Crystal { index: usize, entity: Entity },
    Gate,
}

/// Set by gameplay right before switching to `Screen::Asking`.
#[derive(Resource, Debug, Clone, Copy)]
pub struct AskRequest {
    pub kind: AskKind,
    /// How far through the world the player is, for the difficulty tier.
    pub ratio: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Right,
    Wrong(usize),
    TimeUp,
}

#[derive(Resource)]
pub(super) struct Asking {
    kind: AskKind,
    pub(super) question: Question,
    remaining: f32,
    selected: usize,
    outcome: Option<Outcome>,
    linger: f32,
}

#[derive(Component)]
struct OptionButton(usize);

/// The option clicked with the mouse this frame, if any.
#[derive(Resource, Default)]
struct Clicked(Option<usize>);

#[derive(Component)]
struct TimerBar;

#[derive(Component)]
struct TimerText;

#[derive(Component)]
struct FeedbackArea;

pub fn plugin(app: &mut App) {
    app.init_resource::<Clicked>()
        .add_systems(OnEnter(Screen::Asking), open)
        .add_systems(OnExit(Screen::Asking), |mut commands: Commands| commands.remove_resource::<Asking>())
        .add_systems(
            Update,
            (tick, answer_input, style_options, finish)
                .chain()
                .run_if(in_state(Screen::Asking).and_then(no_overlay).and_then(resource_exists::<Asking>)),
        );
}

fn open(
    mut commands: Commands,
    request: Res<AskRequest>,
    game: Res<Game>,
    mut rng: ResMut<GameRng>,
    assets: Res<GameAssets>,
    rounding: Res<Rounding>,
) {
    let tier = game.0.tier(request.ratio);
    let age = game.0.player().config.age;
    let question = math::generate(age, tier, &mut rng.0);
    debug_assert!(math::validate(&question).is_ok());
    spawn_overlay(&mut commands, &assets, &question, request.kind, rounding.0, &game);
    commands.insert_resource(Asking {
        kind: request.kind,
        question,
        remaining: QUESTION_SECONDS,
        selected: 0,
        outcome: None,
        linger: 0.0,
    });
}

fn spawn_overlay(commands: &mut Commands, a: &GameAssets, q: &Question, kind: AskKind, radius: f32, game: &Game) {
    let heading = match kind {
        AskKind::Crystal { .. } => "Crystal question",
        AskKind::Gate => "Gate key question",
    };
    let who = format!("{} · {}", game.0.player().config.label, game.0.player().config.character.name());

    let root = commands
        .spawn((DespawnOnExit(Screen::Asking), ui::fullscreen(ui::DIM, 60)))
        .id();
    let card = commands
        .spawn((
            Node {
                width: px(760),
                ..ui::card(radius)
            },
            BackgroundColor(ui::PANEL_SOLID),
            BorderColor::all(ui::TEXT),
            AccentBorder,
        ))
        .id();
    commands.entity(root).add_child(card);

    // Header: what kind of question, the topic and stars, and the clock.
    let header = commands
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|h| {
            h.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|c| {
                c.spawn((label(a, heading, 26.0, ui::TEXT), AccentText));
                c.spawn(label(a, format!("{who} · {}", q.topic.label()), 18.0, ui::MUTED));
            });
            h.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                ..default()
            })
            .with_children(|c| {
                c.spawn((Text::new(math::tier_stars(q.tier)), symbol_font(a, 26.0), TextColor(ui::TEXT), AccentText));
                c.spawn((TimerText, label(a, format!("{} s", QUESTION_SECONDS as u32), 22.0, ui::TEXT)));
            });
        })
        .id();

    let bar_bg = commands
        .spawn((
            Node {
                width: percent(100),
                height: px(10),
                border_radius: BorderRadius::all(px(radius.min(5.0))),
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.12)),
        ))
        .with_child((
            TimerBar,
            Node {
                width: percent(100),
                height: percent(100),
                border_radius: BorderRadius::all(px(radius.min(5.0))),
                ..default()
            },
            BackgroundColor(ui::TEXT),
            AccentBackground,
        ))
        .id();

    let prompt = spawn_prompt(commands, a, &q.prompt);

    // Options in a 2×2 grid, numbered for the keyboard.
    let grid = commands
        .spawn(Node {
            width: percent(100),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
            column_gap: px(16),
            row_gap: px(16),
            margin: UiRect::top(px(8)),
            ..default()
        })
        .id();
    for (i, option) in q.options.iter().enumerate() {
        let button = commands
            .spawn((
                OptionButton(i),
                Button,
                Hovered::default(),
                observe(move |_: On<Activate>, mut clicked: ResMut<Clicked>| clicked.0 = Some(i)),
                Node {
                    padding: UiRect::axes(px(20), px(14)),
                    column_gap: px(18),
                    align_items: AlignItems::Center,
                    border: UiRect::all(px(3)),
                    border_radius: BorderRadius::all(px(radius.min(10.0))),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.06)),
                BorderColor::all(Color::NONE),
            ))
            .with_children(|b| {
                b.spawn(label(a, format!("{}", i + 1), 22.0, ui::MUTED));
                b.spawn(label(a, option.to_string(), 36.0, ui::TEXT));
            })
            .id();
        commands.entity(grid).add_child(button);
    }

    let feedback = commands
        .spawn((
            FeedbackArea,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6),
                min_height: px(64),
                ..default()
            },
        ))
        .with_child(label(a, "Press 1–4, or use the arrow keys and Enter", 18.0, ui::MUTED))
        .id();

    commands.entity(card).add_children(&[header, bar_bg, prompt, grid, feedback]);
}

/// Draws the question itself. Young players get the school column layout with tens in
/// red and units in blue; fractions are stacked.
fn spawn_prompt(commands: &mut Commands, a: &GameAssets, prompt: &Prompt) -> Entity {
    let container = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(18),
            padding: UiRect::vertical(px(10)),
            ..default()
        })
        .id();
    match prompt {
        Prompt::Vertical { top, bottom, plus } => {
            // A carry makes a two-digit answer, so the tens column shows even for 4 + 6.
            let two_columns = *top >= 10 || *bottom >= 10 || (*plus && top + bottom >= 10);
            let column = commands
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    ..default()
                })
                .id();
            let digit_row = |commands: &mut Commands, sign: &str, value: Option<i64>, size: f32| {
                commands
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    })
                    .with_children(|r| {
                        let cell = Node {
                            width: px(54),
                            justify_content: JustifyContent::Center,
                            ..default()
                        };
                        r.spawn(cell.clone()).with_child(label(a, sign.to_string(), size, ui::MUTED));
                        if two_columns {
                            let tens = value.filter(|v| *v >= 10).map(|v| (v / 10).to_string()).unwrap_or_default();
                            r.spawn(cell.clone()).with_child(label(a, tens, size, ui::TENS));
                        }
                        let units = value.map(|v| (v % 10).to_string()).unwrap_or_else(|| "?".into());
                        r.spawn(cell).with_child(label(a, units, size, ui::UNITS));
                    })
                    .id()
            };
            let header = commands
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                })
                .with_children(|r| {
                    let cell = Node {
                        width: px(54),
                        justify_content: JustifyContent::Center,
                        ..default()
                    };
                    r.spawn(cell.clone());
                    if two_columns {
                        r.spawn(cell.clone()).with_child(label(a, "T", 18.0, ui::TENS));
                    }
                    r.spawn(cell).with_child(label(a, "U", 18.0, ui::UNITS));
                })
                .id();
            let first = digit_row(commands, "", Some(*top), 60.0);
            let second = digit_row(commands, if *plus { "+" } else { "−" }, Some(*bottom), 60.0);
            let line = commands
                .spawn((
                    Node {
                        width: px(if two_columns { 162.0 } else { 108.0 }),
                        height: px(4),
                        ..default()
                    },
                    BackgroundColor(ui::TEXT),
                ))
                .id();
            let answer = commands
                .spawn(Node {
                    width: px(54),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_child((label(a, "?", 60.0, ui::TEXT), AccentText))
                .id();
            commands.entity(column).add_children(&[header, first, second, line, answer]);
            commands.entity(container).add_child(column);
        }
        Prompt::Gcd { a: x, b: y } => {
            commands.entity(container).with_child(label(a, format!("GCD({x}, {y}) = ?"), 54.0, ui::TEXT));
        }
        Prompt::Lcm { a: x, b: y } => {
            commands.entity(container).with_child(label(a, format!("LCM({x}, {y}) = ?"), 54.0, ui::TEXT));
        }
        Prompt::Fraction { left, op, right } => {
            let left = stacked_fraction(commands, a, left.num, left.den);
            let sign = commands.spawn(label(a, op.symbol().to_string(), 54.0, ui::MUTED)).id();
            let right = stacked_fraction(commands, a, right.num, right.den);
            let equals = commands.spawn((label(a, "= ?", 54.0, ui::TEXT), AccentText)).id();
            commands.entity(container).add_children(&[left, sign, right, equals]);
        }
    }
    container
}

fn stacked_fraction(commands: &mut Commands, a: &GameAssets, num: i64, den: i64) -> Entity {
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|c| {
            c.spawn(label(a, num.to_string(), 44.0, ui::TEXT));
            c.spawn((
                Node {
                    width: px(64),
                    height: px(4),
                    ..default()
                },
                BackgroundColor(ui::TEXT),
            ));
            c.spawn(label(a, den.to_string(), 44.0, ui::TEXT));
        })
        .id()
}

fn tick(
    time: Res<Time>,
    mut asking: ResMut<Asking>,
    mut bars: Query<&mut Node, With<TimerBar>>,
    mut texts: Query<(&mut Text, &mut TextColor), With<TimerText>>,
) {
    if asking.outcome.is_some() {
        asking.linger += time.delta_secs();
        return;
    }
    asking.remaining = (asking.remaining - time.delta_secs()).max(0.0);
    let left = asking.remaining;
    for mut node in &mut bars {
        node.width = percent(left / QUESTION_SECONDS * 100.0);
    }
    for (mut text, mut color) in &mut texts {
        text.0 = format!("{} s", left.ceil() as u32);
        color.0 = if left <= 10.0 { ui::BAD } else { ui::TEXT };
    }
    if left <= 0.0 {
        asking.outcome = Some(Outcome::TimeUp);
    }
}

fn answer_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut asking: ResMut<Asking>,
    hovers: Query<(&Hovered, &OptionButton), Changed<Hovered>>,
    mut clicked: ResMut<Clicked>,
    mut commands: Commands,
    feedback: Query<Entity, With<FeedbackArea>>,
    assets: Res<GameAssets>,
) {
    if asking.outcome.is_some() {
        clicked.0 = None;
        return;
    }
    let digit_keys = [
        [KeyCode::Digit1, KeyCode::Numpad1],
        [KeyCode::Digit2, KeyCode::Numpad2],
        [KeyCode::Digit3, KeyCode::Numpad3],
        [KeyCode::Digit4, KeyCode::Numpad4],
    ];
    let mut choice = None;
    for (i, ks) in digit_keys.iter().enumerate() {
        if keys.any_just_pressed(*ks) {
            choice = Some(i);
        }
    }
    // Arrows move through the 2×2 grid: left/right within a row, up/down between rows.
    let s = asking.selected;
    if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        asking.selected = if s.is_multiple_of(2) { s + 1 } else { s - 1 };
    }
    if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        asking.selected = if s % 2 == 1 { s - 1 } else { s + 1 };
    }
    if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::ArrowUp, KeyCode::KeyW]) {
        asking.selected = (s + 2) % 4;
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter]) {
        choice = Some(asking.selected);
    }
    for (hovered, button) in &hovers {
        if hovered.get() {
            asking.selected = button.0;
        }
    }
    if let Some(i) = clicked.0.take() {
        choice = Some(i);
    }
    let Some(choice) = choice else { return };
    asking.selected = choice;
    asking.outcome = Some(if asking.question.is_correct(choice) { Outcome::Right } else { Outcome::Wrong(choice) });

    // Replace the hint with the verdict and the worked solution.
    let a = &*assets;
    let q = &asking.question;
    let (verdict, color) = match asking.outcome.unwrap() {
        Outcome::Right => ("Correct!".to_string(), ui::GOOD),
        _ => (format!("Not quite: the answer is {}", q.correct_value()), ui::BAD),
    };
    for area in &feedback {
        commands.entity(area).despawn_children().with_children(|c| {
            c.spawn(label(a, verdict.clone(), 26.0, color));
            c.spawn(label(a, q.explanation.clone(), 18.0, ui::MUTED));
            c.spawn(label(a, "Press Space to continue", 16.0, ui::MUTED));
        });
    }
}

fn style_options(
    asking: Res<Asking>,
    accent: Res<Accent>,
    mut buttons: Query<(&OptionButton, &mut BorderColor, &mut BackgroundColor)>,
) {
    for (button, mut border, mut bg) in &mut buttons {
        let i = button.0;
        let (b, fill) = match asking.outcome {
            None if i == asking.selected => (accent.0, Color::srgba(1.0, 1.0, 1.0, 0.12)),
            None => (Color::NONE, Color::srgba(1.0, 1.0, 1.0, 0.06)),
            Some(_) if i == asking.question.correct => (ui::GOOD, ui::GOOD.with_alpha(0.25)),
            Some(Outcome::Wrong(w)) if w == i => (ui::BAD, ui::BAD.with_alpha(0.25)),
            Some(_) => (Color::NONE, Color::srgba(1.0, 1.0, 1.0, 0.03)),
        };
        *border = BorderColor::all(b);
        bg.0 = fill;
    }
}

/// After the verdict has been read, apply it to the match.
#[allow(clippy::too_many_arguments)]
fn finish(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    asking: Res<Asking>,
    mut game: ResMut<Game>,
    worlds: Res<Worlds>,
    mut grid: ResMut<Grid>,
    assets: Res<GameAssets>,
    crystals: Query<&CrystalBlock>,
    mut toast: ResMut<Toast>,
    mut commands: Commands,
    mut feedback: Query<&mut Children, With<FeedbackArea>>,
    mut texts: Query<&mut Text>,
) {
    let Some(outcome) = asking.outcome else { return };
    // A timeout has no keypress behind it, so show its message once.
    if outcome == Outcome::TimeUp && asking.linger == 0.0 {
        for children in &mut feedback {
            for (n, child) in children.iter().enumerate() {
                if let Ok(mut t) = texts.get_mut(child) {
                    t.0 = if n == 0 {
                        format!("Time's up! The answer is {}. Press Space.", asking.question.correct_value())
                    } else {
                        String::new()
                    };
                }
            }
        }
    }
    // A short pause stops the key that picked the answer from also skipping the verdict.
    let ready = asking.linger > 0.35;
    let pressed = keys.any_just_pressed([KeyCode::Space, KeyCode::Enter, KeyCode::NumpadEnter])
        || mouse.just_pressed(MouseButton::Left);
    if !(ready && pressed) && asking.linger < 8.0 {
        return;
    }
    match (outcome, asking.kind) {
        (Outcome::Right, AskKind::Crystal { index, entity }) => {
            game.0.crystal_solved(index);
            grid.used.insert(index);
            // Swap in the spent block at the cell itself, wherever the bump left it.
            if let Ok(crystal) = crystals.get(entity) {
                commands.spawn((
                    LevelEntity,
                    WorldAssetRoot(assets.used.clone()),
                    Transform::from_translation(cell_center(crystal.cell)),
                ));
            }
            commands.entity(entity).despawn();
            toast.show(format!("+{SCORE_CRYSTAL} · +{GEMS_PER_CRYSTAL} gems"));
            commands.set_state(Screen::Running);
        }
        (Outcome::Right, AskKind::Gate) => after_gate(&mut commands, &mut game, &worlds),
        (Outcome::Wrong(_), _) => after_death(&mut commands, &mut game, &worlds, "Wrong answer!"),
        (Outcome::TimeUp, _) => after_death(&mut commands, &mut game, &worlds, "Time's up!"),
    }
}
