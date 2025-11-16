//! Player sprite animation.
//! This is based on multiple examples and may be very different for your game.
//! - [Sprite flipping](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_flipping.rs)
//! - [Sprite animation](https://github.com/bevyengine/bevy/blob/latest/examples/2d/sprite_animation.rs)
//! - [Timers](https://github.com/bevyengine/bevy/blob/latest/examples/time/timers.rs)

use bevy::prelude::*;
use bevy_tnua::{
    TnuaAction,
    builtins::TnuaBuiltinJumpState,
    prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController},
};
use rand::prelude::*;
use std::time::Duration;

use crate::{
    AppSystems, PausableSystems,
    audio::sound_effect,
    game::player::{Dead, PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    // Animate and play sound effects based on controls.
    app.add_systems(
        Update,
        (
            update_animation_timer.in_set(AppSystems::TickTimers),
            (
                handle_animating,
                update_animation_atlas,
                trigger_death_sound_effect,
            )
                .chain()
                .in_set(AppSystems::Update),
        )
            .in_set(PausableSystems),
    );
}

/// Update the animation timer.
fn update_animation_timer(time: Res<Time>, mut query: Query<&mut Animation>) {
    for mut animation in &mut query {
        animation.update_timer(time.delta());
    }
}

/// Update the texture atlas to reflect changes in the animation.
fn update_animation_atlas(mut query: Query<(&Animation, &mut Sprite)>) {
    for (animation, mut sprite) in &mut query {
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        if animation.changed() {
            atlas.index = animation.get_atlas_index();
        }
    }
}

fn trigger_death_sound_effect(
    mut commands: Commands,
    player_assets: If<Res<PlayerAssets>>,
    query: Query<(), Added<Dead>>,
) {
    for _ in query {
        let death = player_assets.death.clone();
        commands.spawn((Name::new("Death Sound"), sound_effect(death)));
    }
}

/// Component that tracks player's animation state.
/// It is tightly bound to the texture atlas we use.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Animation {
    timer: Timer,
    frame: usize,
    current: usize,
    animations: Vec<AnimationData>,
    finished: bool,
}

#[derive(Reflect)]
pub struct AnimationData {
    pub frames: usize,
    pub interval: Duration,
    pub state: AnimationState,
    pub atlas_index: usize,
    pub repeat: Repeat,
}

#[derive(Clone, Copy, Reflect, PartialEq)]
pub enum AnimationState {
    Walking,
    Idle,
    Falling,
    Jumping,
    Dying,
}

#[derive(Clone, Copy, Reflect, PartialEq)]
pub enum Repeat {
    OneShot,
    Loop,
}

impl Animation {
    pub fn new(animations: Vec<AnimationData>) -> Self {
        Self {
            timer: Timer::new(animations[0].interval, TimerMode::Repeating),
            frame: 0,
            current: 0,
            animations,
            finished: false,
        }
    }

    /// Update animation timers.
    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.is_finished() {
            return;
        }
        if self.animations[self.current].repeat == Repeat::Loop {
            self.frame = (self.frame + 1) % self.animations[self.current].frames;
        } else if self.frame + 1 >= self.animations[self.current].frames {
            self.finished = true;
        } else {
            self.frame += 1;
        }
    }

    /// Update animation state if it changes.
    pub fn update_state(&mut self, state: AnimationState) {
        if self.state() != state {
            self.current = self
                .animations
                .iter()
                .position(|a| a.state == state)
                .unwrap();

            let data = &self.animations[self.current];

            self.finished = false;
            self.timer = Timer::new(data.interval, TimerMode::Repeating);
            self.frame = 0;
            self.update_timer(self.timer.remaining());
        }
    }

    /// Whether animation changed this tick.
    pub fn changed(&self) -> bool {
        self.timer.is_finished()
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn state(&self) -> AnimationState {
        self.animations[self.current].state
    }

    /// Return sprite index in the atlas.
    pub fn get_atlas_index(&self) -> usize {
        self.animations[self.current].atlas_index + self.frame
    }
}

fn handle_animating(mut player_query: Query<(&TnuaController, &mut Animation, Has<Dead>)>) {
    let Ok((controller, mut player_animation, is_dead)) = player_query.single_mut() else {
        return;
    };

    if is_dead {
        player_animation.update_state(AnimationState::Dying);
        return;
    }

    let current_status_for_animating = match controller.action_name() {
        Some(TnuaBuiltinJump::NAME) => {
            let (_, jump_state) = controller
                .concrete_action::<TnuaBuiltinJump>()
                .expect("action name mismatch");
            match jump_state {
                TnuaBuiltinJumpState::NoJump => return,
                TnuaBuiltinJumpState::StartingJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::SlowDownTooFastSlopeJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::MaintainingJump { .. } => AnimationState::Jumping,
                TnuaBuiltinJumpState::StoppedMaintainingJump => AnimationState::Jumping,
                TnuaBuiltinJumpState::FallSection => AnimationState::Falling,
            }
        }
        Some(other) => unreachable!("Unknown action {other}"),
        None => {
            // If there is no action going on, we'll base the animation on the state of the
            // basis.
            let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
                return;
            };
            if basis_state.standing_on_entity().is_none() {
                AnimationState::Falling
            } else {
                let speed = basis_state.running_velocity;
                if 0.01 < speed.length() {
                    AnimationState::Walking
                } else {
                    AnimationState::Idle
                }
            }
        }
    };

    player_animation.update_state(current_status_for_animating);
}
