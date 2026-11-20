//! Running and jumping: input, tile physics, enemies, pickups, and the camera.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;

use super::animation::{Animated, Motion, walk_speed};
use super::banner::after_death;
use super::hud::Toast;
use super::sound::{PlaySfx, Sfx};
use super::level::{
    Bob, Body, Bump, CheckpointFlag, CrystalBlock, Drift, Enemy, EnemyModel, GemPickup, Grid, KeyPickup, MainCamera,
    Player, PlayerModel, Spin, facing_rotation,
};
use super::overlay::no_overlay;
use super::question::{AskKind, AskRequest};
use super::{Game, Language, Screen, Worlds};

const GRAVITY: f32 = 34.0;
const JUMP_SPEED: f32 = 14.6;
const MAX_FALL: f32 = 22.0;
const RUN_SPEED: f32 = 6.2;
const GROUND_ACCEL: f32 = 48.0;
const AIR_ACCEL: f32 = 30.0;
const COYOTE: f32 = 0.1;
const JUMP_BUFFER: f32 = 0.12;
const STOMP_BOUNCE: f32 = 10.0;
const ENEMY_SPEED: f32 = 1.5;
const ENEMY_WAKE_DISTANCE: f32 = 16.0;
const PIT_DEPTH: f32 = -4.0;
const EPS: f32 = 0.001;
/// Heroes are drawn a little larger than their collision box, so they read clearly.
pub const HERO_SCALE: f32 = 1.25;
/// How long a Grumble-caught hero lies on screen: the 1.25 s death clip and a beat
/// to take it in, before the death banner.
const DEATH_SECONDS: f32 = 1.6;

/// A Grumble caught the hero: play stops while the death clip plays, then the usual
/// death banner follows. Removed when the banner takes over.
#[derive(Resource, Debug, Default)]
pub struct Dying {
    elapsed: f32,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (player_control, enemy_ai, interactions)
                .chain()
                .run_if(in_state(Screen::Running).and_then(no_overlay).and_then(not(resource_exists::<Dying>))),
            dying.run_if(in_state(Screen::Running).and_then(no_overlay).and_then(resource_exists::<Dying>)),
            animate,
            camera_follow.run_if(resource_exists::<Game>),
        )
            .run_if(not(in_state(Screen::Menu))),
    )
    // Leaving play mid-death (quitting to the menu) must not carry the death over.
    .add_systems(OnExit(Screen::Running), |mut commands: Commands| commands.remove_resource::<Dying>());
}

#[derive(Default, Debug)]
pub struct Contacts {
    pub head: Vec<(i32, i32)>,
    pub wall: bool,
    pub landed: bool,
}

/// Moves a body through the tile grid one axis at a time, so it slides along walls
/// and floors instead of sticking to them.
pub fn move_body(pos: &mut Vec3, body: &mut Body, dt: f32, grid: &Grid) -> Contacts {
    let mut contacts = Contacts::default();
    let size = body.half * 2.0;

    pos.x += body.vel.x * dt;
    for (col, row) in cells(pos, size) {
        if grid.solid(col, row) {
            if body.vel.x > 0.0 {
                pos.x = col as f32 - body.half.x - EPS;
            } else if body.vel.x < 0.0 {
                pos.x = col as f32 + 1.0 + body.half.x + EPS;
            }
            body.vel.x = 0.0;
            contacts.wall = true;
        }
    }
    pos.x = pos.x.max(body.half.x);
    pos.x = pos.x.min(grid.width as f32 - body.half.x);

    pos.y += body.vel.y * dt;
    let mut landed = false;
    for (col, row) in cells(pos, size) {
        if grid.solid(col, row) {
            if body.vel.y > 0.0 {
                pos.y = row as f32 - size.y - EPS;
                contacts.head.push((col, row));
            } else {
                pos.y = row as f32 + 1.0;
                landed = true;
            }
        }
    }
    if landed || !contacts.head.is_empty() {
        body.vel.y = 0.0;
    }
    contacts.landed = landed;
    body.on_ground = landed;
    contacts
}

/// The tile cells a body at `pos` (feet) with `size` overlaps.
fn cells(pos: &Vec3, size: Vec2) -> Vec<(i32, i32)> {
    let min = Vec2::new(pos.x - size.x / 2.0, pos.y);
    let max = Vec2::new(pos.x + size.x / 2.0, pos.y + size.y);
    let mut out = Vec::new();
    for col in min.x.floor() as i32..=(max.x - EPS).floor() as i32 {
        for row in min.y.floor() as i32..=(max.y - EPS).floor() as i32 {
            out.push((col, row));
        }
    }
    out
}

fn move_toward(current: f32, target: f32, step: f32) -> f32 {
    if (target - current).abs() <= step { target } else { current + step * (target - current).signum() }
}

#[allow(clippy::too_many_arguments)]
fn player_control(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    grid: Res<Grid>,
    mut player: Query<(&mut Transform, &mut Body, &mut Player)>,
    mut crystals: Query<(Entity, &CrystalBlock, &Transform), Without<Player>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs().min(1.0 / 30.0);
    let Ok((mut tf, mut body, mut p)) = player.single_mut() else { return };

    let left = keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]);
    let right = keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]);
    let jump_keys = [KeyCode::Space, KeyCode::ArrowUp, KeyCode::KeyW];
    let dir = (right as i32 - left as i32) as f32;

    let accel = if body.on_ground { GROUND_ACCEL } else { AIR_ACCEL };
    body.vel.x = move_toward(body.vel.x, dir * RUN_SPEED, accel * dt);
    if dir != 0.0 {
        p.facing = dir;
    }

    p.coyote = if body.on_ground { COYOTE } else { (p.coyote - dt).max(0.0) };
    p.jump_buffer = if keys.any_just_pressed(jump_keys) { JUMP_BUFFER } else { (p.jump_buffer - dt).max(0.0) };
    if p.jump_buffer > 0.0 && p.coyote > 0.0 {
        body.vel.y = JUMP_SPEED;
        commands.write_message(PlaySfx(Sfx::Jump));
        p.jump_buffer = 0.0;
        p.coyote = 0.0;
        p.squash = -0.25;
    }

    // Releasing jump early cuts the arc short, so small hops are easy for small hands.
    let mut gravity = GRAVITY;
    if body.vel.y > 0.0 && !keys.any_pressed(jump_keys) {
        gravity *= 2.2;
    }
    body.vel.y = (body.vel.y - gravity * dt).max(-MAX_FALL);

    let was_airborne = !body.on_ground;
    let mut pos = tf.translation;
    let contacts = move_body(&mut pos, &mut body, dt, &grid);
    tf.translation = pos;
    if contacts.landed && was_airborne {
        p.squash = 0.2;
    }

    // Bumping a crystal from below asks a question. Pick the one nearest the head.
    if let Some(&(col, row)) = contacts
        .head
        .iter()
        .filter(|(c, r)| grid.crystals.get(&(*c, *r)).is_some_and(|i| !grid.used.contains(i)))
        .min_by(|a, b| {
            let da = (a.0 as f32 + 0.5 - pos.x).abs();
            let db = (b.0 as f32 + 0.5 - pos.x).abs();
            da.total_cmp(&db)
        })
    {
        for (entity, crystal, ctf) in &mut crystals {
            if crystal.cell.col == col && crystal.cell.row == row {
                commands.entity(entity).insert(Bump { base_y: ctf.translation.y, t: 0.0 });
                commands.insert_resource(AskRequest {
                    kind: AskKind::Crystal { index: crystal.index, entity },
                    ratio: pos.x / grid.width as f32,
                });
                commands.set_state(Screen::Asking);
            }
        }
    }
}

fn enemy_ai(
    time: Res<Time>,
    grid: Res<Grid>,
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(Entity, &mut Transform, &mut Body, &mut Enemy, &Children), Without<Player>>,
    mut models: Query<(&mut Transform, &mut Animated), (With<EnemyModel>, Without<Player>, Without<Enemy>)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs().min(1.0 / 30.0);
    let px = player.single().map(|t| t.translation.x).unwrap_or(0.0);
    for (entity, mut tf, mut body, mut enemy, children) in &mut enemies {
        let squashed = enemy.squashed.is_some();
        let awake = !squashed && (tf.translation.x - px).abs() <= ENEMY_WAKE_DISTANCE;
        for child in children.iter() {
            if let Ok((mut m, mut animated)) = models.get_mut(child) {
                m.rotation = facing_rotation(enemy.dir);
                animated.motion = if awake { Motion::Walk } else { Motion::Idle };
                // A stomped Grumble freezes mid-pose while it flattens.
                animated.speed = if squashed { 0.0 } else { 1.0 };
            }
        }
        if let Some(t) = enemy.squashed.as_mut() {
            *t -= dt;
            tf.scale = Vec3::new(1.3, 0.25, 1.3);
            if *t <= 0.0 {
                commands.entity(entity).despawn();
            }
            continue;
        }
        if !awake {
            continue;
        }
        body.vel.x = enemy.dir * ENEMY_SPEED;
        body.vel.y = (body.vel.y - GRAVITY * dt).max(-MAX_FALL);
        let mut pos = tf.translation;
        let contacts = move_body(&mut pos, &mut body, dt, &grid);
        tf.translation = pos;

        // Turn at walls and at ledges, so Grumbles patrol instead of falling off.
        let ahead = (pos.x + enemy.dir * (body.half.x + 0.05)).floor() as i32;
        let below = (pos.y - 0.1).floor() as i32;
        if contacts.wall || (body.on_ground && !grid.solid(ahead, below)) {
            enemy.dir = -enemy.dir;
        }
        if pos.y < PIT_DEPTH {
            commands.entity(entity).despawn();
        }
    }
}

fn overlap(a_pos: Vec3, a_half: Vec2, b_pos: Vec3, b_half: Vec2) -> bool {
    (a_pos.x - b_pos.x).abs() < a_half.x + b_half.x
        && (a_pos.y + a_half.y - (b_pos.y + b_half.y)).abs() < a_half.y + b_half.y
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn interactions(
    mut commands: Commands,
    mut game: ResMut<Game>,
    worlds: Res<Worlds>,
    grid: Res<Grid>,
    mut toast: ResMut<Toast>,
    mut player: Query<(&Transform, &mut Body, &mut Player)>,
    mut enemies: Query<(&Transform, &Body, &mut Enemy), Without<Player>>,
    gems: Query<(Entity, &Transform, &GemPickup), Without<Player>>,
    flags: Query<(&Transform, &CheckpointFlag), Without<Player>>,
    key: Query<&Transform, (With<KeyPickup>, Without<Player>)>,
    language: Res<Language>,
) {
    let Ok((ptf, mut pbody, mut hero)) = player.single_mut() else { return };
    let pos = ptf.translation;
    let t = language.t();

    if pos.y < PIT_DEPTH {
        commands.write_message(PlaySfx(Sfx::Fall));
        after_death(&mut commands, t, &mut game, &worlds, t.fell);
        return;
    }

    for (etf, ebody, mut enemy) in &mut enemies {
        if enemy.squashed.is_some() || !overlap(pos, pbody.half, etf.translation, ebody.half) {
            continue;
        }
        let falling_onto = pbody.vel.y < 0.0 && pos.y > etf.translation.y + ebody.half.y * 0.8;
        if falling_onto {
            enemy.squashed = Some(0.35);
            pbody.vel.y = STOMP_BOUNCE;
            game.0.stomped();
            commands.write_message(PlaySfx(Sfx::Stomp));
            toast.show("+100");
        } else {
            // Freeze here and let the death clip play; `dying` hands over to the banner.
            pbody.vel = Vec2::ZERO;
            hero.knocked_out = true;
            commands.write_message(PlaySfx(Sfx::Caught));
            commands.insert_resource(Dying::default());
            return;
        }
    }

    let center = pos + Vec3::Y * pbody.half.y;
    for (entity, gtf, gem) in &gems {
        if center.distance(gtf.translation) < 0.7 && game.0.gem_collected(gem.index) {
            commands.entity(entity).despawn();
            commands.write_message(PlaySfx(Sfx::Gem));
        }
    }

    for (ftf, flag) in &flags {
        if pos.x >= ftf.translation.x && game.0.checkpoint_reached(flag.index) {
            commands.write_message(PlaySfx(Sfx::Checkpoint));
            toast.show(t.checkpoint);
        }
    }

    if let Ok(ktf) = key.single()
        && center.distance(ktf.translation) < 0.9
    {
        pbody.vel = Vec2::ZERO;
        commands.insert_resource(AskRequest {
            kind: AskKind::Gate,
            ratio: (pos.x / grid.width as f32).min(1.0),
        });
        commands.set_state(Screen::Asking);
    }
}

/// Counts down the death clip, keeping the Grumbles still, then applies the death.
fn dying(
    time: Res<Time>,
    mut state: ResMut<Dying>,
    mut commands: Commands,
    mut game: ResMut<Game>,
    worlds: Res<Worlds>,
    language: Res<Language>,
    mut enemies: Query<&mut Animated, With<EnemyModel>>,
) {
    for mut animated in &mut enemies {
        animated.motion = Motion::Idle;
    }
    state.elapsed += time.delta_secs();
    if state.elapsed >= DEATH_SECONDS {
        commands.remove_resource::<Dying>();
        let t = language.t();
        after_death(&mut commands, t, &mut game, &worlds, t.grumble);
    }
}

#[allow(clippy::type_complexity)]
fn animate(
    time: Res<Time>,
    mut spinners: Query<(&mut Transform, &Spin, Option<&Bob>), Without<PlayerModel>>,
    mut bobbers: Query<(&mut Transform, &Bob), (Without<Spin>, Without<PlayerModel>)>,
    mut drifting: Query<(&mut Transform, &Drift), (Without<Spin>, Without<Bob>, Without<PlayerModel>)>,
    mut bumps: Query<(Entity, &mut Transform, &mut Bump), (Without<Spin>, Without<Bob>, Without<Drift>, Without<PlayerModel>)>,
    mut players: Query<(&mut Player, &Body, &Children)>,
    mut models: Query<
        (&mut Transform, Option<&mut Animated>),
        (With<PlayerModel>, Without<Spin>, Without<Bob>, Without<Drift>, Without<Bump>),
    >,
    screen: Res<State<Screen>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    for (mut tf, spin, bob) in &mut spinners {
        tf.rotate_y(spin.0 * dt);
        if let Some(bob) = bob {
            tf.translation.y = bob.base_y + (t * 2.5 + bob.phase).sin() * 0.1;
        }
    }
    for (mut tf, bob) in &mut bobbers {
        tf.translation.y = bob.base_y + (t * 2.5 + bob.phase).sin() * 0.1;
    }
    for (mut tf, drift) in &mut drifting {
        tf.translation.x += drift.0 * dt;
    }
    for (entity, mut tf, mut bump) in &mut bumps {
        bump.t += dt;
        let k = (bump.t / 0.25).min(1.0);
        tf.translation.y = bump.base_y + (k * std::f32::consts::PI).sin() * 0.3;
        if k >= 1.0 {
            tf.translation.y = bump.base_y;
            commands.entity(entity).remove::<Bump>();
        }
    }
    // Outside of play (a question, a banner) the hero stands still.
    let running = *screen.get() == Screen::Running;
    for (mut p, body, children) in &mut players {
        p.squash = move_toward(p.squash, 0.0, dt * 1.6);
        // A caught hero stays down, behind the banner too, until the level is rebuilt.
        let (motion, speed) = if p.knocked_out {
            (Motion::Death, 1.0)
        } else if !running {
            (Motion::Idle, 1.0)
        } else if !body.on_ground {
            (Motion::Jump, 1.0)
        } else if body.vel.x.abs() > 0.4 {
            (Motion::Walk, walk_speed(body.vel.x, RUN_SPEED))
        } else {
            (Motion::Idle, 1.0)
        };
        for child in children.iter() {
            if let Ok((mut m, animated)) = models.get_mut(child) {
                let target = facing_rotation(p.facing);
                m.rotation = m.rotation.slerp(target, (dt * 14.0).min(1.0));
                // The clips animate the body; landing still squashes the whole model.
                let s = p.squash;
                m.scale = Vec3::new(1.0 + s * 0.5, 1.0 - s, 1.0 + s * 0.5) * HERO_SCALE;
                if let Some(mut animated) = animated {
                    animated.motion = motion;
                    animated.speed = speed;
                }
            }
        }
    }
}

fn camera_follow(
    time: Res<Time<Real>>,
    grid: Res<Grid>,
    player: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(ptf) = player.single() else { return };
    let Ok(mut cam) = camera.single_mut() else { return };
    let max_x = (grid.width as f32 - 8.0).max(8.0);
    let target_x = (ptf.translation.x + 2.0).clamp(6.0, max_x);
    let target_y = (ptf.translation.y * 0.4 + 3.6).clamp(3.6, 6.0);
    let target = Vec3::new(target_x, target_y + 1.5, 13.0);
    let k = (time.delta_secs() * 5.0).min(1.0);
    // Snap when a level is rebuilt far away, glide otherwise.
    if cam.translation.distance(target) > 25.0 {
        cam.translation = target;
    } else {
        cam.translation = cam.translation.lerp(target, k);
    }
    let look = Vec3::new(cam.translation.x, cam.translation.y - 1.6, 0.0);
    cam.look_at(look, Vec3::Y);
}
