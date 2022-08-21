use std::ops::{Range, RangeInclusive};

use bevy::{prelude::*, sprite::Anchor};
use rand::{seq::SliceRandom, Rng};

trait BodyPartMeta {
    fn add_to_stats(&self, stats: &mut Stats);
}

impl BodyPartMeta for () {
    fn add_to_stats(&self, stats: &mut Stats) {}
}

#[derive(Clone)]
struct PartStats {
    skills: Vec<Skill>,
    material: Material,
    weight: f32,

    health: f32,
    energy: f32,

    size: f32,
    color: Color,
}

#[derive(Clone)]
struct BodyPart<M: BodyPartMeta> {
    name: String,
    stats: PartStats,
    meta: M,
}

impl<M: BodyPartMeta> BodyPart<M> {
    fn add_to_stats(&self, stats: &mut Stats) {
        stats.add_part_stats(&self.stats);
        self.meta.add_to_stats(stats);
    }
}

#[derive(Clone)]
struct HeadMeta {
    refresh_rate: f32,
    close_vision: f32,
    far_vision: f32,
}

impl BodyPartMeta for HeadMeta {
    fn add_to_stats(&self, stats: &mut Stats) {
        stats.close_accuracy = self.close_vision.max(stats.close_accuracy);
        stats.far_accuracy = self.far_vision.max(stats.far_accuracy);
        stats.reaction_time = self.refresh_rate.min(stats.reaction_time);
    }
}

#[derive(Clone)]
struct LegMeta {
    max_speed: f32,
    jump_force: f32,
}

impl BodyPartMeta for LegMeta {
    fn add_to_stats(&self, stats: &mut Stats) {
        stats.speed = self.max_speed.min(stats.speed);
        stats.jump_force += self.jump_force;
    }
}

#[derive(Clone)]
struct TorsoMeta {
    arm_slots: usize,
    leg_slots: usize,
}

impl BodyPartMeta for TorsoMeta {
    fn add_to_stats(&self, stats: &mut Stats) {}
}

type Torso = BodyPart<TorsoMeta>;

type Head = BodyPart<HeadMeta>;

type Arm = BodyPart<()>;

type Leg = BodyPart<LegMeta>;

#[derive(Component)]
pub struct Body {
    torso: Torso,
    head: Head,
    arms: Vec<Arm>,
    legs: Vec<Leg>,
}

impl Default for Body {
    fn default() -> Self {
        let material = Material::Rust;
        let color = material.color();
        let create_arm = |index| {
            let skills = vec![Skill::BasicMelee(Ability {
                meta: 5.0,
                time: 1.0,
                cooldown: 0.2,
                energy_cost: 3.0,
                limb: Limb::Arm(index),
                name: "Jab".to_string(),
            })];
            Arm {
                name: "Typical Rusty Arm - V0".to_string(),
                stats: PartStats {
                    skills,
                    material,
                    weight: 16.0,
                    health: 1.0,
                    energy: -2.0,
                    size: 1.0,
                    color,
                },
                meta: (),
            }
        };
        let leg = Leg {
            name: "Normal Rusty Leg - V0".to_string(),
            stats: PartStats {
                skills: vec![Skill::WalkForward, Skill::WalkBackward],
                material,
                weight: 26.0,
                health: 5.0,
                energy: -2.0,
                size: 1.0,
                color,
            },
            meta: LegMeta {
                max_speed: 5.0,
                jump_force: 15.0,
            },
        };
        Self {
            torso: Torso {
                name: "Basic Rusty Torso - V0".to_string(),
                stats: PartStats {
                    skills: vec![],
                    material,
                    weight: 50.0,
                    health: 10.0,
                    energy: -12.0,
                    size: 1.0,
                    color,
                },
                meta: TorsoMeta {
                    arm_slots: 2,
                    leg_slots: 2,
                },
            },
            head: Head {
                name: "Ordinary Rusty Head - V0".to_string(),
                stats: PartStats {
                    skills: vec![],
                    material,
                    weight: 12.0,
                    health: 2.0,
                    energy: -4.0,
                    size: 1.0,
                    color,
                },
                meta: HeadMeta {
                    refresh_rate: 1.0,
                    far_vision: 1.0,
                    close_vision: 1.0,
                },
            },
            arms: vec![create_arm(0), create_arm(1)],
            legs: vec![leg; 2],
        }
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub enum Limb {
    Arm(u8),
    Leg(u8),
}

#[derive(Debug, Clone)]
pub struct Ability<T> {
    pub meta: T,
    pub time: f32,
    pub cooldown: f32,
    pub energy_cost: f32,
    pub limb: Limb,
    pub name: String,
}

impl<T> PartialEq for Ability<T> {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Skill {
    WalkBackward,
    WalkForward,
    TurnAround,
    BasicMelee(Ability<f32>),
    BasicRanged(Ability<f32>),
    Scan(Ability<f32>),
}

impl Skill {
    pub fn get_name(&self) -> &str {
        match self {
            Skill::WalkBackward => "Walk backward",
            Skill::WalkForward => "Walk forward",
            Skill::TurnAround => "Turn around",
            Skill::BasicMelee(a) | Skill::BasicRanged(a) | Skill::Scan(a) => &a.name,
        }
    }

    fn order(&self) -> usize {
        match self {
            Skill::WalkBackward => 0,
            Skill::WalkForward => 1,
            Skill::TurnAround => 2,
            Skill::BasicMelee(_) => 3,
            Skill::BasicRanged(_) => 3,
            Skill::Scan(_) => 4,
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct Stats {
    pub health: f32,
    pub energy: f32,

    pub max_health: f32,
    pub max_energy: f32,
    pub weight: f32,
    pub width: f32,
    pub speed: f32,
    pub reaction_time: f32,
    pub close_accuracy: f32,
    pub far_accuracy: f32,
    pub jump_force: f32,
    pub skills: Vec<Skill>,
}

impl Stats {
    fn add_part_stats(&mut self, part_stats: &PartStats) {
        self.max_health += part_stats.health;
        self.max_energy += part_stats.energy;

        self.weight += part_stats.weight;
        self.skills.extend(part_stats.skills.iter().cloned());
    }
}

#[derive(Copy, Clone)]
pub enum Material {
    Wood,
    Stone,
    Plastic,
    Bronze,
    Aluminum,
    Steel,
    Carbon,
    Rust,
}

impl Material {
    const ALL: &[Material] = &[
        Material::Wood,
        Material::Stone,
        Material::Plastic,
        Material::Bronze,
        Material::Aluminum,
        Material::Steel,
        Material::Carbon,
    ];
    fn choose(rng: &mut impl Rng) -> Material {
        *Self::ALL.choose(rng).unwrap()
    }

    fn base_hp(&self) -> f32 {
        match self {
            Material::Wood => 10.0,
            Material::Stone => 13.0,
            Material::Plastic => 5.0,
            Material::Bronze => 24.0,
            Material::Aluminum => 22.0,
            Material::Steel => 30.0,
            Material::Carbon => 30.0,
            Material::Rust => 10.0,
        }
    }

    fn base_energy(&self) -> f32 {
        match self {
            Material::Wood => -10.0,
            Material::Stone => -15.0,
            Material::Plastic => 0.0,
            Material::Bronze => 10.0,
            Material::Aluminum => 2.0,
            Material::Steel => 2.0,
            Material::Carbon => -1.0,
            Material::Rust => 0.0,
        }
    }

    fn density(&self) -> f32 {
        match self {
            Material::Wood => 100.0,
            Material::Stone => 350.0,
            Material::Plastic => 30.0,
            Material::Bronze => 100.0,
            Material::Aluminum => 60.0,
            Material::Steel => 70.0,
            Material::Carbon => 25.0,
            Material::Rust => 70.0,
        }
    }

    fn color(&self) -> Color {
        match self {
            Material::Wood => Color::rgb_u8(202, 164, 114),
            Material::Stone => Color::rgb_u8(136, 140, 141),
            Material::Plastic => Color::rgb_u8(228, 200, 98),
            Material::Bronze => Color::rgb_u8(205, 127, 50),
            Material::Aluminum => Color::rgb_u8(208, 213, 219),
            Material::Steel => Color::rgb_u8(122, 127, 128),
            Material::Carbon => Color::rgb_u8(13, 17, 21),
            Material::Rust => Color::rgb_u8(183, 65, 14),
        }
    }
}

const ADJECTIVES: &[&str] = &[
    "Rad",
    "Cool",
    "Examplar",
    "Energetic",
    "Clean",
    "Dirty",
    "Hard",
    "Excited",
    "Tiny",
    "Long",
    "Wide",
    "Good",
    "Bad",
    "Sour",
    "Salty",
    "Bitter",
    "Sweet",
    "Spicy",
    "Hot",
    "Swollen",
    "Rational",
    "Decent",
    "Brave",
    "Wise",
    "Glowing",
    "Fair",
    "Sharp",
    "Cowardly",
    "Rude",
    "Clumsy",
    "Stingy",
    "Loyal",
    "Adorable",
    "Beautiful",
    "Awesome",
    "Wonderful",
    "Friendly",
    "Calm",
    "Fresh",
    "Smelly",
    "Stinky",
    "Noisy",
    "Soft",
    "Dull",
    "Blurry",
    "Colorful",
    "Uncomfortable",
    "Turbo",
    "Bussin",
    "Suspicous",
    "Sussy",
];

fn gen_name(rng: &mut impl Rng, part_name: &str) -> String {
    let mut name = format!("{} {} - ", ADJECTIVES.choose(rng).unwrap(), part_name);

    let pre_letters = rng.gen_range(1..=4);
    let numbers = rng.gen_range(2..=5);
    let post_letters = rng.gen_range(0..=1);

    for _ in 0..pre_letters {
        name.push(rng.gen_range('A'..='Z'));
    }
    for _ in 0..numbers {
        name.push(rng.gen_range('0'..='9'));
    }
    for _ in 0..post_letters {
        name.push(rng.gen_range('A'..='Z'));
    }
    name
}

fn randomize_part(
    rng: &mut impl Rng,
    skills: Vec<Skill>,
    density_range: RangeInclusive<f32>,
    hp_mul: f32,
    energy_mul: f32,
) -> PartStats {
    let size = rng.gen_range(0.5..=2.0);
    let material = Material::choose(rng);
    let density = material.density() * rng.gen_range(density_range);
    let weight = size * density;

    let health = material.base_hp() * rng.gen_range(0.2..=5.0f32).powf(0.3) * size.sqrt() * hp_mul;

    let energy =
        material.base_energy() * rng.gen_range(0.2..=5.0f32).powf(0.3) * size.sqrt() * energy_mul;

    let color = randomize_color(material.color(), rng, 0.04);

    PartStats {
        skills,
        material,
        weight,
        health,
        energy,
        size,
        color,
    }
}

fn random_head(rng: &mut impl Rng) -> Head {
    let part_name = ["head", "skull", "noggin"].choose(rng).unwrap();
    Head {
        name: gen_name(rng, part_name),
        stats: randomize_part(rng, vec![], 0.6..=1.0, 0.1, 0.3),
        meta: HeadMeta {
            refresh_rate: rng.gen_range(0.1..=1.0f32).powi(2),
            close_vision: rng.gen_range(0.1..=1.0f32).powi(2),
            far_vision: rng.gen_range(0.1..=1.0f32).powi(2),
        },
    }
}

fn random_arm(rng: &mut impl Rng, i: u8) -> Arm {
    let skills = vec![Skill::BasicMelee(Ability {
        meta: rng.gen_range(100.0..=1000.0f32).sqrt(),
        time: rng.gen_range(0.5..=1.5),
        cooldown: rng.gen_range(0.0..=0.5f32).powi(2),
        energy_cost: rng.gen_range(1.0..=4.0f32).powi(2),
        limb: Limb::Arm(i),
        name: "Jab".to_string(),
    })];

    let part_name = ["arm", "grabber", "limb"].choose(rng).unwrap();
    Arm {
        name: gen_name(rng, part_name),
        stats: randomize_part(rng, skills, 0.6..=1.0, 0.1, 0.3),
        meta: (),
    }
}

fn random_leg(rng: &mut impl Rng) -> Leg {
    let mut skills = vec![Skill::WalkForward, Skill::TurnAround];

    if rng.gen_bool(0.95) {
        skills.push(Skill::WalkBackward);
    }

    let part_name = ["leg", "thigh", "walker"].choose(rng).unwrap();
    Leg {
        name: gen_name(rng, part_name),
        stats: randomize_part(rng, skills, 0.6..=1.0, 0.3, 0.7),
        meta: LegMeta {
            max_speed: rng.gen_range(0.2..=5.0f32).powf(0.2) * rng.gen_range(5.0..=15.0),
            jump_force: rng.gen_range(0.2..=5.0f32).powf(0.2) * rng.gen_range(20.0..=25.0),
        },
    }
}

fn randomize_color(color: Color, rng: &mut impl Rng, amount: f32) -> Color {
    let mut i = color
        .as_rgba_f32()
        .into_iter()
        .map(|c| (c + rng.gen_range(-amount..=amount)).clamp(0.0, 1.0));
    Color::rgb(i.next().unwrap(), i.next().unwrap(), i.next().unwrap())
}

fn random_torso(rng: &mut impl Rng) -> Torso {
    let part_name = ["torso", "body", "trunk", "thorax", "midsection"]
        .choose(rng)
        .unwrap();

    Torso {
        name: gen_name(rng, part_name),
        stats: randomize_part(rng, vec![], 0.8..=1.2, 1.0, 1.0),
        meta: TorsoMeta {
            arm_slots: 2,
            leg_slots: 2,
        },
    }
}

pub fn random_body(rng: &mut impl Rng) -> Body {
    let torso = random_torso(rng);
    let head = random_head(rng);

    let min_arms = (torso.meta.arm_slots as f32 * 0.2).ceil() as usize;
    let max_arms = torso.meta.arm_slots;
    let num_arms = rng.gen_range(min_arms..=max_arms);
    let arms = (0..num_arms as u8).map(|i| random_arm(rng, i)).collect();

    let legs = (0..torso.meta.leg_slots).map(|_| random_leg(rng)).collect();

    Body {
        torso,
        head,
        arms,
        legs,
    }
}

fn update_body_system(
    mut commands: Commands,
    mut bodies: Query<(Entity, &Body, &mut Stats), Changed<Body>>,
) {
    for (entity, body, mut stats) in bodies.iter_mut() {
        let stats = &mut *stats;
        *stats = Stats::default();

        stats.speed = f32::INFINITY;

        stats.max_energy = 100.0;

        body.torso.add_to_stats(stats);
        body.head.add_to_stats(stats);

        for leg in &body.legs {
            leg.add_to_stats(stats);
        }
        for arm in &body.arms {
            arm.add_to_stats(stats);
        }
        stats.skills.sort_by_key(|skill| skill.order());
        stats.skills.dedup();

        commands.entity(entity).despawn_descendants();
        commands.entity(entity).add_children(|parent| {
            let root = Vec3::new(0.0, 0.7, 0.0);
            let torso_scale = Vec3::new(0.3, 1.0, 1.0) * body.torso.stats.size;
            stats.width = torso_scale.x;
            parent
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: body.torso.stats.color,
                        anchor: Anchor::BottomCenter,
                        ..default()
                    },
                    transform: Transform::from_translation(root).with_scale(torso_scale),
                    ..default()
                });
            parent
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: body.head.stats.color,
                        anchor: Anchor::BottomCenter,
                        ..default()
                    },
                    transform: Transform::from_translation(
                        root + Vec3::new(0.0, torso_scale.y, 0.0),
                    )
                    .with_scale(Vec3::splat(body.head.stats.size * 0.5)),
                    ..default()
                });

            for (i, leg) in body.legs.iter().enumerate() {
                let p = (i as f32 / (body.legs.len() - 1) as f32 * torso_scale.x
                    - torso_scale.x / 2.0)
                    * 0.8;
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: leg.stats.color,
                            anchor: Anchor::TopCenter,
                            ..default()
                        },
                        transform: Transform::from_translation(root + Vec3::new(p, 0.0, 0.0))
                            .with_scale(Vec3::new(leg.stats.size * 0.2, root.y, 1.0)),
                        ..default()
                    })
                    .insert(Limb::Leg(i as u8));
            }

            for (i, arm) in body.arms.iter().enumerate() {
                let x = ((i % 2) as f32 * 2.0 - 1.0) * torso_scale.x / 2.0;
                let y =
                    torso_scale.y * (1.0 - (i / 2) as f32 * 2.0 / ((body.legs.len()) - 1) as f32);
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: arm.stats.color,
                            anchor: if i % 2 == 0 {
                                Anchor::TopRight
                            } else {
                                Anchor::TopLeft
                            },
                            ..default()
                        },
                        transform: Transform::from_translation(root + Vec3::new(x, y, 0.0))
                            .with_scale(Vec3::new(arm.stats.size * 0.15, 0.8, 1.0)),
                        ..default()
                    })
                    .insert(Limb::Arm(i as u8));
            }
        });

        stats.health = stats.max_health;
        stats.energy = stats.max_energy;
    }
}

#[derive(Bundle, Default)]
pub struct BodyBundle {
    pub body: Body,
    pub stats: Stats,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

pub struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_body_system);
    }
}
