//! Builds a world in 3D from its domain layout, for the player whose turn it is.
//!
//! Units: one tile is one unit. The play plane is z = 0, the camera looks from +z.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::collections::{HashMap, HashSet};

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

use crate::domain::levels::{Cell, Decor, Level, Tile};
use crate::domain::rng::Rng;
use crate::domain::rules::Character;

use super::animation::{Animated, Rig};
use super::assets::GameAssets;
use super::{Game, Worlds};

/// Everything that belongs to the current level; despawned on rebuild.
#[derive(Component)]
pub struct LevelEntity;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct Sun;

#[derive(Component, Default)]
pub struct Player {
    pub facing: f32,
    pub coyote: f32,
    pub jump_buffer: f32,
    pub squash: f32,
    /// Caught by a Grumble: the death clip plays and holds until the level is rebuilt.
    pub knocked_out: bool,
}

/// The child entity that carries the hero model, so it can turn and squash freely.
#[derive(Component)]
pub struct PlayerModel;

/// An axis-aligned body. The entity's translation is the bottom center (the feet).
#[derive(Component, Default)]
pub struct Body {
    pub vel: Vec2,
    pub half: Vec2,
    pub on_ground: bool,
}

#[derive(Component)]
pub struct Enemy {
    pub dir: f32,
    pub squashed: Option<f32>,
}

#[derive(Component)]
pub struct EnemyModel;

#[derive(Component)]
pub struct CrystalBlock {
    pub index: usize,
    pub cell: Cell,
}

/// A short upward hop when a block is bumped from below.
#[derive(Component)]
pub struct Bump {
    pub base_y: f32,
    pub t: f32,
}

#[derive(Component)]
pub struct GemPickup {
    pub index: usize,
}

#[derive(Component)]
pub struct KeyPickup;

#[derive(Component)]
pub struct CheckpointFlag {
    pub index: usize,
}

#[derive(Component)]
pub struct Spin(pub f32);

#[derive(Component)]
pub struct Bob {
    pub base_y: f32,
    pub phase: f32,
}

#[derive(Component)]
pub struct Drift(pub f32);

/// Solid tiles for collision, with crystal identities.
#[derive(Resource, Default)]
pub struct Grid {
    pub solids: HashMap<(i32, i32), Tile>,
    pub crystals: HashMap<(i32, i32), usize>,
    pub used: HashSet<usize>,
    pub width: i32,
    pub world: usize,
}

impl Grid {
    pub fn solid(&self, col: i32, row: i32) -> bool {
        self.solids.contains_key(&(col, row))
    }
}

/// Ask for the level to be rebuilt for the current player and their world.
#[derive(Resource)]
pub struct PendingBuild;

pub fn plugin(app: &mut App) {
    app.init_resource::<Grid>()
        .add_systems(Startup, spawn_camera_and_sun)
        .add_systems(PreUpdate, rebuild.run_if(resource_exists::<PendingBuild>.and_then(resource_exists::<Game>)));
}

fn spawn_camera_and_sun(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        Transform::from_xyz(4.0, 5.0, 14.0).looking_at(Vec3::new(4.0, 3.2, 0.0), Vec3::Y),
        DistanceFog {
            color: Color::srgb(0.70, 0.86, 0.98),
            falloff: FogFalloff::Linear { start: 30.0, end: 90.0 },
            ..default()
        },
    ));
    commands.spawn((
        Sun,
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-6.0, 14.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 2,
            first_cascade_far_bound: 18.0,
            maximum_distance: 45.0,
            ..default()
        }
        .build(),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 600.0,
        ..default()
    });
}

/// Hero model handle for a character.
pub fn hero_model(assets: &GameAssets, character: Character) -> Handle<WorldAsset> {
    match character {
        Character::Pip => assets.hero_pip.clone(),
        Character::Bolt => assets.hero_bolt.clone(),
    }
}

/// Turns a hero so it faces its direction but shows a little more face to the camera.
pub fn facing_rotation(facing: f32) -> Quat {
    if facing >= 0.0 {
        Quat::from_rotation_y(-0.5)
    } else {
        Quat::from_rotation_y(std::f32::consts::PI + 0.5)
    }
}

pub fn cell_center(c: Cell) -> Vec3 {
    Vec3::new(c.col as f32 + 0.5, c.row as f32 + 0.5, 0.0)
}

pub fn cell_floor(c: Cell) -> Vec3 {
    Vec3::new(c.col as f32 + 0.5, c.row as f32, 0.0)
}

#[allow(clippy::too_many_arguments)]
fn rebuild(
    mut commands: Commands,
    game: Res<Game>,
    worlds: Res<Worlds>,
    assets: Res<GameAssets>,
    old: Query<Entity, With<LevelEntity>>,
    mut grid: ResMut<Grid>,
    mut clear: ResMut<ClearColor>,
    mut fog: Query<&mut DistanceFog, With<MainCamera>>,
    mut sun: Query<&mut DirectionalLight, With<Sun>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.remove_resource::<PendingBuild>();
    for e in &old {
        commands.entity(e).despawn();
    }
    let player = game.0.player();
    let world = player.world.min(worlds.0.len() - 1);
    let level = &worlds.0[world];

    let p = level.palette;
    clear.0 = Color::srgb(p.sky[0], p.sky[1], p.sky[2]);
    for mut f in &mut fog {
        f.color = Color::srgb(p.fog[0], p.fog[1], p.fog[2]);
    }
    for mut s in &mut sun {
        s.color = Color::srgb(p.sun[0], p.sun[1], p.sun[2]);
        s.illuminance = p.sun_lux;
    }

    *grid = build_grid(level, world, &player.progress.used_crystals);
    spawn_level(
        &mut commands,
        &assets,
        level,
        world,
        &player.progress,
        player.config.character,
        &mut meshes,
        &mut materials,
    );
}

fn build_grid(level: &Level, world: usize, used: &HashSet<usize>) -> Grid {
    let mut grid = Grid {
        width: level.width,
        world,
        used: used.clone(),
        ..default()
    };
    for (cell, tile) in &level.tiles {
        grid.solids.insert((cell.col, cell.row), *tile);
    }
    for (i, cell) in level.crystals().into_iter().enumerate() {
        grid.crystals.insert((cell.col, cell.row), i);
    }
    grid
}

#[allow(clippy::too_many_arguments)]
fn spawn_level(
    commands: &mut Commands,
    assets: &GameAssets,
    level: &Level,
    world: usize,
    progress: &crate::domain::rules::WorldProgress,
    character: Character,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let crystal_ids: HashMap<Cell, usize> = level.crystals().into_iter().enumerate().map(|(i, c)| (c, i)).collect();

    // Only the top ground tile of each column shows grass; soil fills in below it,
    // deep enough that the bottom of the screen never shows sky.
    let ground: HashSet<Cell> = level.tiles.iter().filter(|(_, t)| *t == Tile::Ground).map(|(c, _)| *c).collect();
    let soil_mesh = meshes.add(Cuboid::new(1.0, 1.0, 3.0));
    let soil_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.52, 0.35, 0.24),
        perceptual_roughness: 1.0,
        ..default()
    });
    const SOIL_BOTTOM: f32 = -10.0;
    for (cell, tile) in &level.tiles {
        match tile {
            Tile::Ground => {
                if ground.contains(&Cell { col: cell.col, row: cell.row + 1 }) {
                    continue;
                }
                commands.spawn((
                    LevelEntity,
                    WorldAssetRoot(assets.ground.clone()),
                    Transform::from_xyz(cell.col as f32 + 0.5, cell.row as f32 + 1.0, 0.0),
                ));
                let height = cell.row as f32 - SOIL_BOTTOM;
                commands.spawn((
                    LevelEntity,
                    Mesh3d(soil_mesh.clone()),
                    MeshMaterial3d(soil_material.clone()),
                    Transform::from_xyz(cell.col as f32 + 0.5, SOIL_BOTTOM + height / 2.0, 0.0)
                        .with_scale(Vec3::new(1.0, height, 1.0)),
                ));
            }
            Tile::Block => {
                commands.spawn((LevelEntity, WorldAssetRoot(assets.brick.clone()), Transform::from_translation(cell_center(*cell))));
            }
            Tile::Crystal => {
                let index = crystal_ids[cell];
                let used = progress.used_crystals.contains(&index);
                let model = if used { assets.used.clone() } else { assets.crystal.clone() };
                let mut e = commands.spawn((LevelEntity, WorldAssetRoot(model), Transform::from_translation(cell_center(*cell))));
                if !used {
                    e.insert(CrystalBlock { index, cell: *cell });
                }
            }
        }
    }

    for (index, cell) in level.gems.iter().enumerate() {
        if progress.collected_gems.contains(&index) {
            continue;
        }
        let pos = cell_center(*cell);
        commands.spawn((
            LevelEntity,
            GemPickup { index },
            WorldAssetRoot(assets.gem.clone()),
            Transform::from_translation(pos).with_scale(Vec3::splat(1.3)),
            Spin(2.0),
            Bob { base_y: pos.y, phase: index as f32 },
        ));
    }

    for cell in &level.enemies {
        commands
            .spawn((
                LevelEntity,
                Enemy { dir: -1.0, squashed: None },
                Body {
                    half: Vec2::new(0.36, 0.3),
                    ..default()
                },
                Transform::from_translation(cell_floor(*cell)),
                Visibility::default(),
            ))
            .with_child((
                EnemyModel,
                Animated::new(Rig::Grumble),
                WorldAssetRoot(assets.enemy.clone()),
                Transform::from_rotation(facing_rotation(-1.0)).with_scale(Vec3::splat(1.1)),
            ));
    }

    for (index, cell) in level.checkpoints.iter().enumerate() {
        commands.spawn((
            LevelEntity,
            CheckpointFlag { index },
            WorldAssetRoot(assets.flag.clone()),
            Transform::from_translation(cell_floor(*cell) + Vec3::new(0.0, 0.0, -0.6)),
        ));
    }

    for (cell, decor) in &level.decor {
        let model = match decor {
            Decor::Tree => assets.tree.clone(),
            Decor::Bush => assets.bush.clone(),
        };
        commands.spawn((
            LevelEntity,
            WorldAssetRoot(model),
            Transform::from_translation(cell_floor(*cell) + Vec3::new(0.0, 0.0, -1.0)),
        ));
    }

    let key_pos = cell_center(level.key);
    commands.spawn((
        LevelEntity,
        KeyPickup,
        WorldAssetRoot(assets.key.clone()),
        Transform::from_translation(key_pos).with_scale(Vec3::splat(1.4)),
        Spin(1.5),
        Bob { base_y: key_pos.y, phase: 0.0 },
    ));
    commands.spawn((
        LevelEntity,
        WorldAssetRoot(assets.gate.clone()),
        Transform::from_translation(cell_floor(level.gate) + Vec3::new(0.0, 0.0, -0.4)),
    ));

    spawn_backdrop(commands, assets, level, world, meshes, materials);

    // The player last, at the start or their latest checkpoint.
    let spawn = cell_floor(level.respawn(progress.checkpoint));
    commands
        .spawn((
            LevelEntity,
            Player { facing: 1.0, ..default() },
            Body {
                half: Vec2::new(0.28, 0.42),
                ..default()
            },
            Transform::from_translation(spawn),
            Visibility::default(),
        ))
        .with_child((
            PlayerModel,
            Animated::new(Rig::hero(character)),
            WorldAssetRoot(hero_model(assets, character)),
            Transform::from_rotation(facing_rotation(1.0)).with_scale(Vec3::splat(super::play::HERO_SCALE)),
        ));
}

/// Scenery behind the play strip: a valley floor, trees at a few depths, and clouds.
/// Positions come from a seed per world so a level always looks the same.
fn spawn_backdrop(
    commands: &mut Commands,
    assets: &GameAssets,
    level: &Level,
    world: usize,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mut rng = Rng::new(1000 + world as u64);
    let floor_color = if world == 0 { Color::srgb(0.38, 0.62, 0.30) } else { Color::srgb(0.42, 0.34, 0.42) };
    let width = level.width as f32 + 60.0;
    commands.spawn((
        LevelEntity,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(width, 60.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: floor_color,
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(level.width as f32 / 2.0, 0.6, -31.6),
    ));

    let mut x = -6.0;
    while x < level.width as f32 + 10.0 {
        let z = -rng.range(30, 90) as f32 / 10.0;
        let scale = rng.range(80, 140) as f32 / 100.0;
        let model = if rng.chance(0.7) { assets.tree.clone() } else { assets.bush.clone() };
        commands.spawn((
            LevelEntity,
            WorldAssetRoot(model),
            Transform::from_xyz(x, 0.6, z)
                .with_scale(Vec3::splat(scale * 1.4))
                .with_rotation(Quat::from_rotation_y(rng.range(0, 628) as f32 / 100.0)),
        ));
        x += rng.range(20, 50) as f32 / 10.0;
    }

    let mut x = -10.0;
    while x < level.width as f32 + 20.0 {
        let y = rng.range(85, 120) as f32 / 10.0;
        let z = -rng.range(100, 200) as f32 / 10.0;
        commands.spawn((
            LevelEntity,
            WorldAssetRoot(assets.cloud.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(rng.range(100, 180) as f32 / 100.0)),
            Drift(rng.range(2, 6) as f32 / 10.0),
        ));
        x += rng.range(80, 160) as f32 / 10.0;
    }
}
