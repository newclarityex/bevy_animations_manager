use bevy::{prelude::*, sprite::Anchor};
use std::{collections::HashMap, time::Duration};

#[derive(Clone)]
pub struct AnimationData {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub frame_count: usize,
    pub frame_durations: Vec<u64>,
    pub anchor: Anchor,
}

#[derive(Event)]
pub struct AnimationFrameEvent {
    pub entity: Entity,
    pub animation: String,
    pub frame: usize,
}

#[derive(Event)]
pub struct AnimationCompleteEvent {
    pub entity: Entity,
    pub animation: String,
}

#[derive(Event)]
pub struct AnimationLoopEvent {
    pub entity: Entity,
    pub animation: String,
}

#[derive(Debug, Clone)]
struct InvalidAnimationError;

#[derive(Component, Clone)]
pub struct AnimationsManager {
    timer: Timer,
    pub animation_map: HashMap<String, AnimationData>,
    pub current_animation: Option<String>,
    pub paused: bool,
    pub index: usize,
    pub looping: bool,
    pub time_scale: f64,
}
impl AnimationsManager {
    pub fn new() -> Self {
        AnimationsManager {
            paused: false,
            timer: Timer::from_seconds(0., TimerMode::Repeating),
            animation_map: HashMap::new(),
            current_animation: None,
            index: 0,
            looping: false,
            time_scale: 1.,
        }
    }

    fn update_timer_duration(&mut self) {
        let Some(current_animation_data) = self.get_current() else {
            return;
        };
        let duration_ms = current_animation_data
            .frame_durations
            .get(self.index)
            .expect("Duration missing for frame!");

        self.timer.set_duration(Duration::from_millis(*duration_ms));
    }

    pub fn load_animation<S: Into<String>>(
        &mut self,
        new_animation: S,
        animation_data: AnimationData,
    ) {
        self.animation_map
            .insert(new_animation.into(), animation_data);
    }

    // Play animation from beginning
    pub fn play<S: Into<String>>(&mut self, new_animation: S) {
        let new_animation = new_animation.into();
        self.animation_map
            .get(&new_animation)
            .expect("Can't play animation that isn't loaded!");

        self.current_animation = Some(new_animation);
        self.update_timer_duration();
    }

    pub fn clear(&mut self) {
        self.index = 0;
        self.timer.set_elapsed(Duration::ZERO);
        self.timer.set_duration(Duration::ZERO);
        self.current_animation = None;
    }

    // Set animation if it's not already running
    pub fn set_animation<S: Into<String>>(&mut self, new_animation: S) {
        let new_animation = new_animation.into();

        match &self.current_animation {
            Some(current_animation) => {
                if *current_animation != new_animation {
                    self.play(new_animation);
                }
            }
            None => {
                self.play(new_animation);
            }
        }
    }

    pub fn get_current(&self) -> Option<&AnimationData> {
        if let Some(current_animation) = &self.current_animation {
            self.animation_map.get(current_animation)
        } else {
            None
        }
    }
}

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationCompleteEvent>()
            .add_event::<AnimationLoopEvent>()
            .add_event::<AnimationFrameEvent>()
            .add_systems(Update, update_animations);
    }
}

fn update_animations(
    mut ev_complete: EventWriter<AnimationCompleteEvent>,
    mut ev_loop: EventWriter<AnimationLoopEvent>,
    mut ev_frame: EventWriter<AnimationFrameEvent>,
    mut query: Query<(
        Entity,
        &mut Sprite,
        &mut TextureAtlas,
        &mut Handle<Image>,
        &mut AnimationsManager,
    )>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut texture_atlas, mut texture, mut animations_manager) in
        query.iter_mut()
    {
        if animations_manager.paused {
            continue;
        }

        let scaled_elapsed =
            Duration::from_secs_f64(time.delta_seconds_f64() * animations_manager.time_scale);

        animations_manager.timer.tick(scaled_elapsed);

        let Some(current_animation) = animations_manager.current_animation.clone() else {
            continue;
        };

        let Some(animation_data) = animations_manager.get_current() else {
            continue;
        };

        sprite.anchor = animation_data.anchor;

        *texture = animation_data.texture.clone();
        texture_atlas.layout = animation_data.layout.clone();
        texture_atlas.index = animations_manager.index;
        sprite.anchor = animation_data.anchor;

        if animations_manager.timer.just_finished() {
            let frame_count = animation_data.frame_count;

            animations_manager.index += 1;
            if animations_manager.index == frame_count {
                if animations_manager.looping {
                    animations_manager.index = 0;
                    ev_loop.send(AnimationLoopEvent {
                        entity,
                        animation: current_animation.clone(),
                    });
                } else {
                    animations_manager.clear();
                    ev_complete.send(AnimationCompleteEvent {
                        entity,
                        animation: current_animation.clone(),
                    });
                }
            }

            ev_frame.send(AnimationFrameEvent {
                entity,
                animation: current_animation.clone(),
                frame: animations_manager.index,
            });

            animations_manager.update_timer_duration();
        }
    }
}
