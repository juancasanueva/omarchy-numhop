//! The start menu: one or two players, each with a name, an age level and a hero.
//! Choices are remembered between matches, and names between runs.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use crate::domain::math::AgeLevel;
use crate::domain::rules::{Character, MAX_NAME_CHARS, Match, PlayerConfig, clean_name, name_char_allowed};
use crate::i18n::{Strings, fill};

use super::animation::{Animated, Rig};
use super::assets::GameAssets;
use super::banner::turn_banner;
use super::level::{LevelEntity, MainCamera, PendingBuild, facing_rotation, hero_model};
use super::overlay::no_overlay;
use super::theme::{Accent, AccentBorder, AccentText, Rounding};
use super::ui::{self, label};
use super::{Game, Language, PlayerNames, Screen, Worlds};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Page {
    #[default]
    Main,
    Settings,
    About,
}

#[derive(Resource, Debug, Clone)]
pub struct MenuChoices {
    pub players: usize,
    /// Ages and heroes; each player's label is written in the current language
    /// when the match starts.
    pub configs: [PlayerConfig; 2],
    row: usize,
    page: Page,
    /// The player whose name is being typed, and the text so far.
    editing: Option<(usize, String)>,
}

impl Default for MenuChoices {
    fn default() -> Self {
        Self {
            players: 1,
            configs: [
                PlayerConfig {
                    label: String::new(),
                    age: AgeLevel::Six,
                    character: Character::Pip,
                },
                PlayerConfig {
                    label: String::new(),
                    age: AgeLevel::Twelve,
                    character: Character::Bolt,
                },
            ],
            row: 0,
            page: Page::Main,
            editing: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Players,
    Name(usize),
    Age(usize),
    Hero(usize),
    About,
    Settings,
    Start,
    Language,
    Back,
}

impl MenuChoices {
    fn rows(&self) -> Vec<Row> {
        match self.page {
            Page::Main => {
                let mut rows = vec![Row::Players, Row::Name(0), Row::Age(0), Row::Hero(0)];
                if self.players == 2 {
                    rows.extend([Row::Name(1), Row::Age(1), Row::Hero(1)]);
                }
                rows.extend([Row::About, Row::Settings, Row::Start]);
                rows
            }
            Page::Settings => vec![Row::Language, Row::Back],
            // The About page is read-only: its lines are drawn above the one button.
            Page::About => vec![Row::Back],
        }
    }

    fn open(&mut self, page: Page, focus: Row) {
        self.page = page;
        self.row = self.rows().iter().position(|r| *r == focus).unwrap_or(0);
    }

    /// True while a player is typing a name, so letter keys type instead of acting.
    pub fn typing(&self) -> bool {
        self.editing.is_some()
    }
}

/// The About page: the version, where the code lives, and the licenses. The version,
/// repository and license come from Cargo.toml, so they never drift from the package.
fn about_lines(t: &Strings) -> [(&'static str, String); 5] {
    let repo = env!("CARGO_PKG_REPOSITORY");
    [
        (t.about_version, env!("CARGO_PKG_VERSION").to_string()),
        (t.about_source, repo.trim_start_matches("https://").to_string()),
        (t.about_license, env!("CARGO_PKG_LICENSE").to_string()),
        (t.about_fonts, "Noto · Apache 2.0".to_string()),
        (t.about_inspired, "github.com/pgarevalo/mariomath".to_string()),
    ]
}

/// The name a player goes by: what they typed, or "Player 1" when they typed nothing.
pub fn shown_name(t: &Strings, names: &PlayerNames, index: usize) -> String {
    let typed = &names.0[index];
    if typed.is_empty() { t.player(index) } else { typed.clone() }
}

#[derive(Component)]
struct MenuUi;

#[derive(Component)]
struct MenuHero(usize);

pub fn plugin(app: &mut App) {
    app.init_resource::<MenuChoices>()
        .add_systems(OnEnter(Screen::Menu), (enter, draw).chain())
        .add_systems(
            Update,
            (navigate, draw.run_if(menu_changed), spin_heroes)
                .chain()
                .run_if(in_state(Screen::Menu).and_then(no_overlay)),
        );
}

/// The menu redraws when a choice, the language, or a name changes.
fn menu_changed(choices: Res<MenuChoices>, language: Res<Language>, names: Res<PlayerNames>) -> bool {
    choices.is_changed() || language.is_changed() || names.is_changed()
}

fn enter(
    mut commands: Commands,
    old: Query<Entity, With<LevelEntity>>,
    assets: Res<GameAssets>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut clear: ResMut<ClearColor>,
) {
    commands.remove_resource::<Game>();
    commands.remove_resource::<PendingBuild>();
    for e in &old {
        commands.entity(e).despawn();
    }
    clear.0 = Color::srgb(0.53, 0.78, 0.96);
    if let Ok(mut cam) = camera.single_mut() {
        *cam = Transform::from_xyz(0.0, 2.2, 7.5).looking_at(Vec3::new(0.0, 1.4, 0.0), Vec3::Y);
    }
    // A little stage of ground for the heroes to stand on, with some scenery.
    for x in -8..=8 {
        commands.spawn((
            DespawnOnExit(Screen::Menu),
            WorldAssetRoot(assets.ground.clone()),
            Transform::from_xyz(x as f32, 0.0, 0.0),
        ));
    }
    for (x, z, model) in [(-4.5, -2.0, &assets.tree), (4.8, -2.5, &assets.tree), (-2.6, -1.0, &assets.bush), (2.8, -1.2, &assets.bush)] {
        commands.spawn((
            DespawnOnExit(Screen::Menu),
            WorldAssetRoot(model.clone()),
            Transform::from_xyz(x, 0.0, z),
        ));
    }
    commands.spawn((
        DespawnOnExit(Screen::Menu),
        WorldAssetRoot(assets.cloud.clone()),
        Transform::from_xyz(-3.0, 5.0, -6.0),
    ));
    commands.spawn((
        DespawnOnExit(Screen::Menu),
        WorldAssetRoot(assets.crystal.clone()),
        Transform::from_xyz(0.0, 3.6, -1.0).with_scale(Vec3::splat(0.8)),
    ));
}

fn draw(
    mut commands: Commands,
    choices: Res<MenuChoices>,
    assets: Res<GameAssets>,
    rounding: Res<Rounding>,
    accent: Res<Accent>,
    language: Res<Language>,
    names: Res<PlayerNames>,
    old_ui: Query<Entity, With<MenuUi>>,
    old_heroes: Query<Entity, With<MenuHero>>,
) {
    for e in old_ui.iter().chain(old_heroes.iter()) {
        commands.entity(e).despawn();
    }
    let a = &*assets;
    let t = language.t();

    // Heroes on the stage reflect the current picks.
    let positions: &[f32] = if choices.players == 1 { &[-1.6] } else { &[-2.2, -0.9] };
    for (i, x) in positions.iter().enumerate() {
        commands.spawn((
            MenuHero(i),
            Animated::new(Rig::hero(choices.configs[i].character)),
            DespawnOnExit(Screen::Menu),
            WorldAssetRoot(hero_model(a, choices.configs[i].character)),
            Transform::from_xyz(*x, 0.0, 1.0).with_scale(Vec3::splat(1.6)).with_rotation(facing_rotation(1.0)),
        ));
    }

    let rows = choices.rows();
    let card = commands
        .spawn((
            Node {
                min_width: px(470),
                ..ui::card(rounding.0)
            },
            BackgroundColor(ui::PANEL),
            BorderColor::all(ui::TEXT),
            AccentBorder,
        ))
        .with_children(|c| {
            c.spawn((label(a, "numhop", 64.0, ui::TEXT), AccentText));
            let subtitle = match choices.page {
                Page::Main => t.tagline,
                Page::Settings => t.settings,
                Page::About => t.about,
            };
            c.spawn(label(a, subtitle, 20.0, ui::MUTED));
        })
        .id();

    if choices.page == Page::About {
        for (name, value) in about_lines(t) {
            let line = commands
                .spawn(Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    column_gap: px(30),
                    // Rows have a 2px border, so pad 2px more to line up with them.
                    padding: UiRect::axes(px(18), px(8)),
                    ..default()
                })
                .with_children(|r| {
                    r.spawn(label(a, name, 20.0, ui::MUTED));
                    r.spawn(label(a, value, 20.0, ui::TEXT));
                })
                .id();
            commands.entity(card).add_child(line);
        }
    }
    for (i, row) in rows.iter().enumerate() {
        let selected = i == choices.row;
        let (name, value) = match row {
            Row::Players => (t.players.to_string(), choices.players.to_string()),
            Row::Name(p) => {
                let value = match &choices.editing {
                    // A cursor shows that the keyboard is typing into this row.
                    Some((editing, draft)) if editing == p => format!("{draft}_"),
                    _ if names.0[*p].is_empty() => t.name_placeholder.to_string(),
                    _ => names.0[*p].clone(),
                };
                (t.player(*p), value)
            }
            Row::Age(p) => (
                fill(t.player_level, &[&shown_name(t, &names, *p)]),
                t.age(choices.configs[*p].age).to_string(),
            ),
            Row::Hero(p) => (
                fill(t.player_hero, &[&shown_name(t, &names, *p)]),
                choices.configs[*p].character.name().to_string(),
            ),
            Row::About => (t.about.to_string(), String::new()),
            Row::Settings => (t.settings.to_string(), String::new()),
            Row::Start => (t.start.to_string(), String::new()),
            Row::Language => (t.language.to_string(), language.0.native_name().to_string()),
            Row::Back => (t.back.to_string(), String::new()),
        };
        // The main action of each page stands out, centered and larger.
        let primary = matches!(row, Row::Start | Row::Back);
        let border = if selected { accent.0 } else { Color::NONE };
        let row_entity = commands
            .spawn((
                Node {
                    width: percent(100),
                    justify_content: if primary { JustifyContent::Center } else { JustifyContent::SpaceBetween },
                    column_gap: px(30),
                    padding: UiRect::axes(px(16), px(10)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(rounding.0.min(8.0))),
                    ..default()
                },
                BorderColor::all(border),
                BackgroundColor(if selected { Color::srgba(1.0, 1.0, 1.0, 0.08) } else { Color::NONE }),
            ))
            .with_children(|r| {
                let size = if primary { 28.0 } else { 20.0 };
                r.spawn(label(a, name, size, if primary { ui::TEXT } else { ui::MUTED }));
                if !value.is_empty() {
                    let arrows = selected && !matches!(row, Row::Name(_));
                    let shown = if arrows { format!("‹  {value}  ›") } else { value };
                    r.spawn(label(a, shown, 20.0, ui::TEXT));
                }
            })
            .id();
        commands.entity(card).add_child(row_entity);
    }
    let hint = match choices.page {
        Page::Main if choices.typing() => t.name_hint,
        Page::Main => t.menu_hint,
        Page::Settings => t.settings_hint,
        Page::About => t.about_hint,
    };
    commands.entity(card).with_child(label(a, hint, 15.0, ui::MUTED));

    commands
        .spawn((
            MenuUi,
            DespawnOnExit(Screen::Menu),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::End,
                align_items: AlignItems::Center,
                padding: UiRect::right(px(70)),
                ..default()
            },
        ))
        .add_child(card);
}

fn navigate(
    keys: Res<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    mut choices: ResMut<MenuChoices>,
    mut language: ResMut<Language>,
    mut names: ResMut<PlayerNames>,
    worlds: Res<Worlds>,
    mut commands: Commands,
) {
    // Read the typed keys every frame, so none are left over when typing starts.
    let typed: Vec<KeyboardInput> = typed.read().cloned().collect();
    if let Some((p, draft)) = choices.editing.clone() {
        // Edit without Bevy's automatic change flag, then flag only real changes: an idle
        // typing frame must neither redraw the menu nor rewrite the settings file.
        let before = (choices.editing.clone(), names.0.clone());
        type_name(&keys, &typed, choices.bypass_change_detection(), names.bypass_change_detection(), p, draft);
        if choices.editing != before.0 {
            choices.set_changed();
        }
        if names.0 != before.1 {
            names.set_changed();
        }
        return;
    }
    let rows = choices.rows();
    let row = rows[choices.row.min(rows.len() - 1)];
    if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::Tab]) {
        choices.row = (choices.row + 1) % rows.len();
    }
    if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        choices.row = (choices.row + rows.len() - 1) % rows.len();
    }
    let left = keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]);
    let right = keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]);
    let confirm = keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);

    if choices.page != Page::Main {
        let back = keys.any_just_pressed([KeyCode::Escape, KeyCode::Backspace]) || (confirm && row == Row::Back);
        if back {
            // Return to the row that opened this page.
            let from = if choices.page == Page::Settings { Row::Settings } else { Row::About };
            choices.open(Page::Main, from);
        } else if row == Row::Language && (left || right || confirm) {
            language.0 = if left { language.0.previous() } else { language.0.next() };
        }
        return;
    }

    let start = keys.just_pressed(KeyCode::Space) || (confirm && row == Row::Start);
    if start {
        let t = language.t();
        let mut configs = choices.configs[..choices.players].to_vec();
        for (i, c) in configs.iter_mut().enumerate() {
            c.label = shown_name(t, &names, i);
        }
        let game = Game(Match::new(configs, worlds.0.len()));
        let banner = turn_banner(t, &game, &worlds, t.ready.into(), None);
        commands.insert_resource(game);
        commands.insert_resource(banner);
        commands.insert_resource(PendingBuild);
        commands.set_state(Screen::Banner);
        return;
    }
    if left || right || confirm {
        match row {
            Row::Players => {
                choices.players = if choices.players == 1 { 2 } else { 1 };
                choices.row = 0;
            }
            Row::Name(p) if confirm || right => choices.editing = Some((p, names.0[p].clone())),
            Row::Age(p) => {
                let age = choices.configs[p].age;
                choices.configs[p].age = if left { age.previous() } else { age.next() };
            }
            Row::Hero(p) => choices.configs[p].character = choices.configs[p].character.next(),
            Row::About if confirm || right => choices.open(Page::About, Row::Back),
            Row::Settings if confirm || right => choices.open(Page::Settings, Row::Language),
            _ => {}
        }
    }
}

/// One frame of typing a name: Enter keeps it, Esc drops it, Backspace deletes, and
/// allowed characters are added up to the length limit.
fn type_name(
    keys: &ButtonInput<KeyCode>,
    typed: &[KeyboardInput],
    choices: &mut MenuChoices,
    names: &mut PlayerNames,
    player: usize,
    mut draft: String,
) {
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        names.0[player] = clean_name(&draft);
        choices.editing = None;
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        choices.editing = None;
        return;
    }
    let before = draft.clone();
    for key in typed.iter().filter(|k| k.state.is_pressed()) {
        if key.key_code == KeyCode::Backspace {
            draft.pop();
            continue;
        }
        for c in key.text.iter().flat_map(|text| text.chars()) {
            if name_char_allowed(c) && draft.chars().count() < MAX_NAME_CHARS {
                draft.push(c);
            }
        }
    }
    // Only touch the menu when the text changed, so it does not redraw every frame.
    if draft != before {
        choices.editing = Some((player, draft));
    }
}

fn spin_heroes(time: Res<Time>, mut heroes: Query<(&mut Transform, &MenuHero)>) {
    let t = time.elapsed_secs();
    for (mut tf, hero) in &mut heroes {
        let wobble = (t * 1.3 + hero.0 as f32 * 1.7).sin() * 0.35;
        tf.rotation = facing_rotation(1.0) * Quat::from_rotation_y(wobble);
        tf.translation.y = ((t * 3.0 + hero.0 as f32).sin().abs()) * 0.12;
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, NativeKeyCode};

    use super::*;

    fn key(text: &str) -> KeyboardInput {
        KeyboardInput {
            key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified),
            logical_key: Key::Character(text.into()),
            state: ButtonState::Pressed,
            text: Some(text.into()),
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    fn backspace() -> KeyboardInput {
        KeyboardInput {
            key_code: KeyCode::Backspace,
            logical_key: Key::Backspace,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    /// Types `keys` into player 0's name, then optionally presses `finish`.
    fn type_into(start: &str, keys: &[KeyboardInput], finish: Option<KeyCode>) -> (MenuChoices, PlayerNames) {
        let mut choices = MenuChoices { editing: Some((0, start.into())), ..default() };
        let mut names = PlayerNames::default();
        let input = ButtonInput::<KeyCode>::default();
        let draft = choices.editing.clone().unwrap().1;
        type_name(&input, keys, &mut choices, &mut names, 0, draft);
        if let Some(k) = finish {
            let mut input = ButtonInput::<KeyCode>::default();
            input.press(k);
            let draft = choices.editing.clone().unwrap().1;
            type_name(&input, &[], &mut choices, &mut names, 0, draft);
        }
        (choices, names)
    }

    #[test]
    fn menu_keys_are_letters_while_typing() {
        // Q quits, Space starts, W/A/S/D move: while typing they are just letters.
        let keys = ["Q", "w", "a", "s", "d", " ", "ñ", "á"].map(key);
        let (choices, _) = type_into("", &keys, None);
        assert_eq!(choices.editing, Some((0, "Qwasd ñá".into())));
    }

    #[test]
    fn backspace_deletes_and_odd_characters_are_skipped() {
        let keys = [key("L"), key("u"), key("\""), key("\\"), key("?"), backspace(), key("z")];
        let (choices, _) = type_into("", &keys, None);
        assert_eq!(choices.editing, Some((0, "Lz".into())));
    }

    #[test]
    fn names_stop_at_the_length_limit() {
        let keys: Vec<_> = (0..20).map(|_| key("x")).collect();
        let (choices, _) = type_into("", &keys, None);
        assert_eq!(choices.editing.unwrap().1.chars().count(), MAX_NAME_CHARS);
    }

    #[test]
    fn enter_keeps_the_cleaned_name_and_escape_drops_it() {
        let (choices, names) = type_into("  Ana  María ", &[], Some(KeyCode::Enter));
        assert!(!choices.typing());
        assert_eq!(names.0[0], "Ana María");

        // Start from a saved name, so the test fails if Esc wiped or replaced it.
        let mut choices = MenuChoices { editing: Some((0, "Lucía".into())), ..default() };
        let mut names = PlayerNames(["Mateo".into(), String::new()]);
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::Escape);
        type_name(&input, &[], &mut choices, &mut names, 0, "Lucía".into());
        assert!(!choices.typing());
        assert_eq!(names.0[0], "Mateo", "Esc keeps the old name");
    }

    /// How many frames saw the menu choices and the names marked as changed.
    #[derive(Resource, Default)]
    struct Changes {
        choices: u32,
        names: u32,
    }

    fn count_changes(choices: Res<MenuChoices>, names: Res<PlayerNames>, mut changes: ResMut<Changes>) {
        changes.choices += choices.is_changed() as u32;
        changes.names += names.is_changed() as u32;
    }

    #[test]
    fn idle_typing_frames_change_nothing() {
        let mut app = App::new();
        app.add_message::<KeyboardInput>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Changes>()
            .insert_resource(MenuChoices { editing: Some((0, "Lu".into())), ..default() })
            .insert_resource(PlayerNames::default())
            .insert_resource(Language(crate::i18n::Lang::En))
            .insert_resource(Worlds(Vec::new()))
            .add_systems(Update, (navigate, count_changes).chain());
        // The first frame counts once, because the resources were just added.
        for _ in 0..4 {
            app.update();
        }
        let changes = app.world().resource::<Changes>();
        assert_eq!((changes.choices, changes.names), (1, 1), "idle frames must not mark changes");

        // A typed letter changes the draft (a redraw) but not the saved names (no save).
        app.world_mut().write_message(key("c"));
        app.update();
        let changes = app.world().resource::<Changes>();
        assert_eq!((changes.choices, changes.names), (2, 1));
        assert_eq!(app.world().resource::<MenuChoices>().editing, Some((0, "Luc".into())));
    }

    #[test]
    fn about_opens_from_the_menu_and_shows_the_package_facts() {
        let mut choices = MenuChoices::default();
        let rows = choices.rows();
        let about = rows.iter().position(|r| *r == Row::About).unwrap();
        assert_eq!(rows[about + 1], Row::Settings, "About sits just above Settings");
        choices.open(Page::About, Row::Back);
        assert_eq!(choices.rows(), vec![Row::Back], "only Back is selectable on About");

        let t = crate::i18n::Lang::En.strings();
        let lines = about_lines(t);
        assert_eq!(lines[0].1, env!("CARGO_PKG_VERSION"));
        assert_eq!(lines[1].1, "github.com/juancasanueva/omarchy-numhop");
        assert_eq!(lines[2].1, "MIT");
    }

    #[test]
    fn an_empty_name_shows_the_player_number() {
        let t = crate::i18n::Lang::En.strings();
        let names = PlayerNames(["".into(), "Mateo".into()]);
        assert_eq!(shown_name(t, &names, 0), "Player 1");
        assert_eq!(shown_name(t, &names, 1), "Mateo");
    }
}
