mod body;
mod ui;

use std::f32::consts::PI;

use bevy::{prelude::*, render::camera::ScalingMode, sprite::Anchor};
use body::{random_body, BodyBundle, Limb, Stats};
use smallmap::Map;
use ui::UseSkill;

struct Game {
    player: Entity,
    enemy: Entity,
}

pub struct BodyTransforms<'a, 'world, 'state, 'inner> {
    transforms: &'a mut Query<'world, 'state, &'inner mut Transform>,
    legs: Map<u8, Entity>,
    arms: Map<u8, Entity>,
}

impl<'a, 'world, 'state, 'inner> BodyTransforms<'a, 'world, 'state, 'inner> {
    fn get_entity(&self, limb: Limb) -> Entity {
        match &limb {
            Limb::Arm(i) => *self.arms.get(i).unwrap(),
            Limb::Leg(i) => *self.legs.get(i).unwrap(),
        }
    }

    pub fn get(&self, limb: Limb) -> &Transform {
        self.transforms.get(self.get_entity(limb)).unwrap()
    }
    pub fn get_mut(&mut self, limb: Limb) -> Mut<Transform> {
        self.transforms.get_mut(self.get_entity(limb)).unwrap()
    }

    pub fn for_legs(&mut self, mut f: impl FnMut(u8, &mut Transform)) {
        for (i, e) in self.legs.iter() {
            f(*i, self.transforms.get_mut(*e).unwrap().as_mut());
        }
    }

    pub fn for_arms(&mut self, mut f: impl FnMut(u8, &mut Transform)) {
        for (i, e) in self.arms.iter() {
            f(*i, self.transforms.get_mut(*e).unwrap().as_mut());
        }
    }
}

struct Animation {
    skill: usize,
    progress: f32,
}

fn do_animation(
    entity: Entity,
    enemy: Entity,
    stats: Query<(&Stats, &Children)>,
    animation: &mut Animation,
    limbs: Query<&Limb>,
    mut transforms: Query<&mut Transform>,
    time: &Time,
) {
    let [(stats, children), (enemy_stats, _)] = stats.get_many([entity, enemy]).unwrap();

    let (mut position, mut direction) = {
        let transform = transforms.get(entity).unwrap();

        (transform.translation.x, transform.scale.x)
    };

    let mut body_parts = BodyTransforms {
        transforms: &mut transforms,
        legs: Map::new(),
        arms: Map::new(),
    };
    for e in children {
        if let Ok(limb) = limbs.get(*e) {
            match limb {
                Limb::Arm(i) => body_parts.arms.insert(*i, *e),
                Limb::Leg(i) => body_parts.legs.insert(*i, *e),
            };
        }
    }

    let dt = time.delta_seconds();

    fn walk(
        position: &mut f32,
        direction: f32,
        mul: f32,
        dt: f32,
        stats: &Stats,
        animation: &Animation,
        body_parts: &mut BodyTransforms,
    ) {
        *position += dt * stats.speed * direction * mul;
        const END_TIME: f32 = 0.1;
        if animation.progress < 1.0 - END_TIME {
            let distance_moved = stats.speed * animation.progress * mul;
            body_parts.for_legs(|i, transform| {
                let sign = (i % 2) as f32 * 2.0 - 1.0;

                let a = (sign * distance_moved).sin();

                transform.rotation = Quat::from_rotation_z(a);
            });
            body_parts.for_arms(|i, transform| {
                let sign = -((i % 2) as f32 * 2.0 - 1.0);

                let a = (sign * distance_moved).sin();

                transform.rotation = Quat::from_rotation_z(a);
            });
        } else {
            let t = (animation.progress - 1.0 + END_TIME) / END_TIME;
            body_parts.for_legs(|_, transform| {
                transform.rotation = transform.rotation.lerp(Quat::IDENTITY, t);
            });
            body_parts.for_arms(|_, transform| {
                transform.rotation = transform.rotation.lerp(Quat::IDENTITY, t);
            });
        }
    }

    match &stats.skills[animation.skill] {
        body::Skill::WalkBackward => {
            walk(
                &mut position,
                direction,
                -0.5,
                dt,
                stats,
                animation,
                &mut body_parts,
            );
        }
        body::Skill::WalkForward => {
            walk(
                &mut position,
                direction,
                1.0,
                dt,
                stats,
                animation,
                &mut body_parts,
            );
        }
        body::Skill::TurnAround => {
            if animation.progress <= 0.5 {
                let t = 1.0 - animation.progress * 2.0;
                direction = direction * t - direction.signum() * 0.0001;
            } else {
                let t = animation.progress * 2.0 - 1.0;
                direction = direction * (1.0 - t) + t * direction.signum();
            }
        }
        body::Skill::BasicMelee(ability) => {
            let mut transform = body_parts.get_mut(ability.limb);

            let a = (animation.progress * PI).sin();
            transform.rotation = Quat::from_rotation_z(a);
        }
        body::Skill::BasicRanged(_) => todo!(),
        body::Skill::Scan(_) => todo!(),
    }

    {
        let [mut transform, enemy] = transforms.get_many_mut([entity, enemy]).unwrap();
        if transform.translation.x < enemy.translation.x {
            transform.translation.x =
                position.min(enemy.translation.x - (stats.width + enemy_stats.width) / 2.0 - 0.1);
        } else {
            transform.translation.x =
                position.max(enemy.translation.x + (stats.width + enemy_stats.width) / 2.0 + 0.1);
        }
        transform.scale.x = direction;
    }

    const ANIMATION_SPEED: f32 = 1.0;
    animation.progress += dt * ANIMATION_SPEED;
}

fn use_skill_system(
    mut use_skill: ResMut<UseSkill>,
    game: Res<Game>,
    time: Res<Time>,
    stats: Query<(&Stats, &Children)>,
    limbs: Query<&Limb>,
    transforms: Query<&mut Transform>,
    mut maybe_animation: Local<Option<Animation>>,
) {
    if let Some(animation) = maybe_animation.as_mut() {
        do_animation(
            game.player,
            game.enemy,
            stats,
            animation,
            limbs,
            transforms,
            &time,
        );
        if animation.progress > 1.0 {
            *maybe_animation = None;
            **use_skill = None;
        }
    } else if let Some(skill) = (*use_skill).as_ref() {
        *maybe_animation = Some(Animation {
            skill: *skill,
            progress: 0.0,
        });
    }
}

fn scene_setup_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        transform: Transform::from_scale(Vec3::splat(5.0))
            .with_translation(Vec3::new(0.0, 0.0, 0.0)),
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal(5.0),
            ..default()
        },
        ..default()
    });

    let size = 40.0;
    commands.spawn_bundle(SpriteBundle {
        transform: Transform::from_scale(Vec3::new(size, size, 1.0)),
        sprite: Sprite {
            color: Color::BLACK,
            anchor: Anchor::TopCenter,
            ..default()
        },
        ..default()
    });
    let player = commands
        .spawn_bundle(BodyBundle {
            // body: random_body(&mut rand::thread_rng()),
            transform: Transform::from_translation(Vec3::new(-4.0, 0.0, 0.0)),
            ..default()
        })
        .id();

    let enemy = commands
        .spawn_bundle(BodyBundle {
            body: random_body(&mut rand::thread_rng()),
            transform: Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)),
            ..default()
        })
        .id();

    commands.insert_resource(Game { player, enemy });
}

fn dynamic_camera(
    game: Res<Game>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    transforms: Query<&Transform, Without<Camera>>,
) {
    let player = game.player;
    let enemy = game.enemy;
    let mut camera_transform = camera_transform.single_mut();
    let player_transform = transforms.get(player).unwrap();
    let enemy_transform = transforms.get(enemy).unwrap();
    let vector_between = enemy_transform.translation - player_transform.translation;
    let distance_between = vector_between.length();
    let look_at_pos = player_transform.translation + vector_between / 2.0;
    camera_transform.translation.x = look_at_pos.x;
    camera_transform.translation.y = look_at_pos.y;
    camera_transform.scale = Vec3::splat(distance_between / 6.0 + 8.0);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ui::UiPlugin)
        .add_plugin(body::BodyPlugin)
        .add_system(bevy::window::close_on_esc)
        .add_startup_system(scene_setup_system)
        .add_system(use_skill_system)
        .add_system(dynamic_camera)
        .run();
}
