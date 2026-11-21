//! Skeletal animation for the characters.
//!
//! Each character's glTF carries named clips made in Blender (`walk`, `idle`, and for
//! heroes `jump` and `death`). This module builds one animation graph per character, wires it to
//! the `AnimationPlayer` the glTF scene spawns, and blends between clips. Gameplay
//! only says what a character is doing by setting [`Animated::motion`] and
//! [`Animated::speed`].

use std::time::Duration;

use bevy::prelude::*;

use crate::domain::rules::Character;

use super::assets::GameAssets;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rig {
    Pip,
    Bolt,
    Fern,
    Grumble,
}

impl Rig {
    pub fn hero(character: Character) -> Self {
        match character {
            Character::Pip => Rig::Pip,
            Character::Bolt => Rig::Bolt,
            Character::Fern => Rig::Fern,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    #[default]
    Idle,
    Walk,
    Jump,
    /// Played once when a Grumble catches a hero; the last pose holds.
    Death,
}

/// What a character model is doing. Put it on the entity that holds the model's
/// `WorldAssetRoot`; the animation player inside the scene is found on its own.
#[derive(Component, Debug)]
pub struct Animated {
    pub rig: Rig,
    pub motion: Motion,
    /// Playback speed of the current clip; walking scales it with running speed.
    pub speed: f32,
    player: Option<Entity>,
    playing: Option<Motion>,
}

impl Animated {
    pub fn new(rig: Rig) -> Self {
        Self {
            rig,
            motion: Motion::Idle,
            speed: 1.0,
            player: None,
            playing: None,
        }
    }
}

struct Clips {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    /// The Grumble never jumps or dies this way, so it has neither clip.
    jump: Option<AnimationNodeIndex>,
    death: Option<AnimationNodeIndex>,
}

impl Clips {
    fn node(&self, motion: Motion) -> AnimationNodeIndex {
        match motion {
            Motion::Idle => self.idle,
            Motion::Walk => self.walk,
            Motion::Jump => self.jump.unwrap_or(self.idle),
            Motion::Death => self.death.unwrap_or(self.idle),
        }
    }
}

/// Animation graphs for every rig, built once the glTF files have loaded.
#[derive(Resource)]
pub struct CharacterClips {
    pip: Clips,
    bolt: Clips,
    fern: Clips,
    grumble: Clips,
}

impl CharacterClips {
    fn get(&self, rig: Rig) -> &Clips {
        match rig {
            Rig::Pip => &self.pip,
            Rig::Bolt => &self.bolt,
            Rig::Fern => &self.fern,
            Rig::Grumble => &self.grumble,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            prepare_clips.run_if(not(resource_exists::<CharacterClips>)),
            (attach_players, play_motions).chain().run_if(resource_exists::<CharacterClips>),
        ),
    );
}

/// Builds the graph for one character from its named clips. Returns `None` while the
/// file is still loading, and logs once if a clip is missing. Heroes need `jump` and
/// `death` too; the Grumble only walks and idles.
fn clips_for(gltf: Option<&Gltf>, graphs: &mut Assets<AnimationGraph>, file: &str, hero: bool) -> Option<Clips> {
    let gltf = gltf?;
    let clip = |name: &str, required: bool| {
        let found = gltf.named_animations.get(name).cloned();
        if found.is_none() && required {
            error!("{file} has no '{name}' animation; re-export it from assets-src/build_models.py");
        }
        found
    };
    let idle = clip("idle", true)?;
    let walk = clip("walk", true)?;
    let jump = clip("jump", hero);
    let death = clip("death", hero);
    if hero && (jump.is_none() || death.is_none()) {
        return None;
    }
    let mut graph = AnimationGraph::new();
    let root = graph.root;
    let idle = graph.add_clip(idle, 1.0, root);
    let walk = graph.add_clip(walk, 1.0, root);
    let jump = jump.map(|j| graph.add_clip(j, 1.0, root));
    let death = death.map(|d| graph.add_clip(d, 1.0, root));
    Some(Clips {
        graph: graphs.add(graph),
        idle,
        walk,
        jump,
        death,
    })
}

/// Set when a character file loaded but lacks its clips, so the loading screen can
/// say so instead of waiting forever.
#[derive(Resource)]
pub struct ClipsMissing;

fn prepare_clips(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    missing: Option<Res<ClipsMissing>>,
) {
    let Some(assets) = assets else { return };
    if missing.is_some() {
        return;
    }
    let handles = [&assets.gltf_pip, &assets.gltf_bolt, &assets.gltf_fern, &assets.gltf_grumble];
    if !handles.iter().all(|h| server.is_loaded_with_dependencies(*h)) {
        return;
    }
    let pip = clips_for(gltfs.get(&assets.gltf_pip), &mut graphs, "hero_pip.glb", true);
    let bolt = clips_for(gltfs.get(&assets.gltf_bolt), &mut graphs, "hero_bolt.glb", true);
    let fern = clips_for(gltfs.get(&assets.gltf_fern), &mut graphs, "hero_fern.glb", true);
    let grumble = clips_for(gltfs.get(&assets.gltf_grumble), &mut graphs, "enemy_grumble.glb", false);
    match (pip, bolt, fern, grumble) {
        (Some(pip), Some(bolt), Some(fern), Some(grumble)) => commands.insert_resource(CharacterClips { pip, bolt, fern, grumble }),
        _ => commands.insert_resource(ClipsMissing),
    }
}

/// Finds the animation player inside each newly spawned character scene and gives it
/// the right graph. Scenes spawn a frame or more after their root, so this retries
/// until the player exists.
fn attach_players(
    mut commands: Commands,
    clips: Res<CharacterClips>,
    mut models: Query<(Entity, &mut Animated)>,
    children: Query<&Children>,
    players: Query<(), With<AnimationPlayer>>,
) {
    for (entity, mut animated) in &mut models {
        if animated.player.is_some() {
            continue;
        }
        let Some(player) = children.iter_descendants(entity).find(|e| players.contains(*e)) else {
            continue;
        };
        commands.entity(player).insert((
            AnimationGraphHandle(clips.get(animated.rig).graph.clone()),
            AnimationTransitions::new(),
        ));
        animated.player = Some(player);
        animated.playing = None;
    }
}

fn play_motions(
    clips: Res<CharacterClips>,
    mut models: Query<&mut Animated>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for mut animated in &mut models {
        let Some(entity) = animated.player else { continue };
        let Ok((mut player, mut transitions)) = players.get_mut(entity) else { continue };
        let set = clips.get(animated.rig);
        let node = set.node(animated.motion);
        if animated.playing != Some(animated.motion) {
            // Take off quickly into a jump or a fall; ease between walking and standing.
            let blend = if matches!(animated.motion, Motion::Jump | Motion::Death) { 80 } else { 180 };
            let active = transitions.play(&mut player, node, Duration::from_millis(blend));
            // Every clip loops except death, which ends on its last pose and stays there.
            if animated.motion != Motion::Death {
                active.repeat();
            }
            animated.playing = Some(animated.motion);
        }
        if let Some(active) = player.animation_mut(node) {
            active.set_speed(animated.speed);
        }
    }
}

/// Clip playback speed for walking at `run_speed` out of `max_speed`: the clip is a
/// one-second stride, sped up so short legs keep pace with the ground.
pub fn walk_speed(run_speed: f32, max_speed: f32) -> f32 {
    0.8 + 1.6 * (run_speed.abs() / max_speed).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clips(death: Option<AnimationNodeIndex>) -> Clips {
        Clips {
            graph: Handle::default(),
            idle: AnimationNodeIndex::new(1),
            walk: AnimationNodeIndex::new(2),
            jump: None,
            death,
        }
    }

    #[test]
    fn death_plays_its_clip_and_falls_back_to_idle_without_one() {
        let hero = clips(Some(AnimationNodeIndex::new(4)));
        assert_eq!(hero.node(Motion::Death), AnimationNodeIndex::new(4));
        let grumble = clips(None);
        assert_eq!(grumble.node(Motion::Death), grumble.idle);
    }

    #[test]
    fn walk_speed_grows_with_running_and_stays_bounded() {
        assert!((walk_speed(0.0, 6.0) - 0.8).abs() < 1e-6);
        assert!((walk_speed(6.0, 6.0) - 2.4).abs() < 1e-6);
        assert!((walk_speed(-6.0, 6.0) - 2.4).abs() < 1e-6);
        assert!((walk_speed(60.0, 6.0) - 2.4).abs() < 1e-6);
    }
}
