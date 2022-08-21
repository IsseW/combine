use std::ops::{Deref, DerefMut};

use bevy::{prelude::*, ui::FocusPolicy};

use crate::{body::{Stats, Skill}, Game};

const NORMAL_BUTTON: Color = Color::rgb(0.75, 0.75, 0.75);
const HOVERED_BUTTON: Color = Color::rgb(1.0, 1.0, 1.0);
const PRESSED_BUTTON: Color = Color::rgb(1.0, 0.75, 0.75);

pub struct Fonts {
    normal: Handle<Font>,
    bold: Handle<Font>,
}

impl Fonts {
    pub fn normal(&self) -> Handle<Font> {
        return self.normal.clone();
    }
    pub fn bold(&self) -> Handle<Font> {
        return self.bold.clone();
    }
}

#[derive(Component)]
struct SkillButton(usize);

struct Hovered {
    entity: Entity,
    header: String,
    description: String,
}

struct Tooltip {
    entity: Entity,
    currently_hovering: Option<Hovered>,
}

#[derive(Default)]
pub struct UseSkill(Option<usize>);

impl Deref for UseSkill {
    type Target = Option<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UseSkill {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn button_system(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut UiColor, &SkillButton),
        Changed<Interaction>,
    >,
    mut use_skill: ResMut<UseSkill>,
    mut tooltip: ResMut<Tooltip>,
    stats_query: Query<&Stats>,
    game: Res<Game>,
) {
    if use_skill.is_none() {
        for (entity, interaction, mut color, skill_button) in &mut interaction_query {
            match *interaction {
                Interaction::Clicked => {
                    **use_skill = Some(skill_button.0);
                    *color = PRESSED_BUTTON.into();
                }
                Interaction::Hovered => {
                    if let Some(skill) = stats_query
                        .get(game.player)
                        .ok()
                        .and_then(|stats| stats.skills.get(skill_button.0))
                    {
                        tooltip.currently_hovering = Some(Hovered {
                            entity,
                            header: skill.get_name().to_string(),
                            description: "Cool Foo You Got There lmaoaoaoaoao".to_string(),
                        });
                    }
                    *color = HOVERED_BUTTON.into();
                }
                Interaction::None => {
                    if tooltip
                        .currently_hovering
                        .as_ref()
                        .map_or(false, |e| e.entity == entity)
                    {
                        tooltip.currently_hovering = None;
                    }
                    *color = NORMAL_BUTTON.into();
                }
            }
        }
    }
}

fn button_disable_system(mut interaction_query: Query<(&mut UiColor, &SkillButton)>, use_skill: Res<UseSkill>, mut tooltip: ResMut<Tooltip>) {
    if use_skill.is_changed() {
        if let Some(skill) = **use_skill {
            tooltip.currently_hovering = None;
            for (mut color, skill_btn) in interaction_query.iter_mut() {
                if skill == skill_btn.0 {
                    color.0 = PRESSED_BUTTON;
                } else {
                    color.0 = Color::rgb(0.1, 0.1, 0.1);
                }
            }
        } else {
            for (mut color, _) in interaction_query.iter_mut() {
                color.0 = NORMAL_BUTTON;
            }
        }
    }
}

fn tooltip_system(
    mut commands: Commands,
    mut tooltip: ResMut<Tooltip>,
    transforms: Query<(&GlobalTransform, &Node)>,
    mut styles: Query<&mut Style>,
    fonts: Res<Fonts>,
) {
    if tooltip.is_changed() {
        let e = tooltip.entity;
        let mut commands = commands.entity(e);
        commands.despawn_descendants();
        if let Some(ref mut hovered) = tooltip.currently_hovering {
            if let (Ok(mut a), Ok([(_, a_n), (b, b_n)])) =
                (styles.get_mut(e), transforms.get_many([e, hovered.entity]))
            {
                a.position = UiRect::new(
                    Val::Px(b.translation().x - a_n.size.x / 2.0),
                    Val::Auto,
                    Val::Auto,
                    Val::Px(b.translation().y + b_n.size.y / 2.0),
                );
            }

            commands.add_children(|commands| {
                commands
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Undefined),
                            ..default()
                        },
                        color: Color::rgba_u8(65, 70, 72, 120).into(),
                        focus_policy: FocusPolicy::Pass,
                        ..default()
                    })
                    .add_children(|commands| {
                        let mut formatted_string = hovered.description.clone();
                        formatted_string.insert(0, '\n');
                        let mut count: usize = 0;
                        let mut save_next = false;
                        let mut char_end = Vec::new();
                        let newline_positions = formatted_string
                            .char_indices()
                            .filter_map(|(i, c)| {
                                if save_next {
                                    char_end.push(i);
                                    save_next = false;
                                }
                                if c == '\n' {
                                    count = 0;
                                    None
                                } else if count > 20 && c.is_whitespace() {
                                    count = 0;
                                    save_next = true;
                                    Some(i)
                                } else {
                                    count += 1;
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        if save_next {
                            char_end.push(formatted_string.len());
                        }

                        for (s, e) in newline_positions.into_iter().zip(char_end) {
                            formatted_string.replace_range(s..e, "\n");
                        }

                        commands.spawn_bundle(TextBundle::from_sections([
                            TextSection::new(
                                &hovered.header,
                                TextStyle {
                                    font: fonts.bold(),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            ),
                            TextSection::new(
                                formatted_string,
                                TextStyle {
                                    font: fonts.normal(),
                                    font_size: 12.0,
                                    color: Color::WHITE,
                                },
                            ),
                        ]));
                    });
            });
        }
    }
}

fn update_ui_system(
    mut commands: Commands,
    game: Res<Game>,
    player: Query<&Stats, Changed<Stats>>,
    asset_server: Res<AssetServer>,
    existing: Local<Option<Entity>>,
) {
    if let Ok(stats) = player.get(game.player) {
        if let Some(existing) = *existing {
            commands.entity(existing).despawn();
        }
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(20.0)),
                    justify_content: JustifyContent::SpaceAround,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                },
                color: Color::NONE.into(),
                ..default()
            })
            .with_children(|parent| {
                for (i, skill) in stats.skills.iter().enumerate() {
                    let image = match skill {
                        Skill::WalkBackward => "textures/arrow_left.png",
                        Skill::WalkForward => "textures/arrow_right.png",
                        Skill::TurnAround => "textures/round_arrow.png",
                        Skill::BasicMelee(_) => "textures/fist.png",
                        Skill::BasicRanged(_) => todo!(),
                        Skill::Scan(_) => todo!(),
                    };
                    parent
                        .spawn_bundle(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                // horizontally center child text
                                justify_content: JustifyContent::Center,
                                // vertically center child text
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            image: UiImage(asset_server.load(image)),
                            color: NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .insert(SkillButton(i));
                }
            });
    }
}

fn ui_startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Fonts {
        normal: asset_server.load("fonts/FiraMono-Medium.ttf"),
        bold: asset_server.load("fonts/FiraSans-Bold.ttf"),
    });

    let hover = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(200.0), Val::Px(200.0)),
                position_type: PositionType::Absolute,
                ..default()
            },
            color: Color::NONE.into(),
            focus_policy: FocusPolicy::Pass,
            ..default()
        })
        .id();

    commands.insert_resource(Tooltip {
        entity: hover,
        currently_hovering: None,
    });
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(update_ui_system)
                .with_system(button_system)
                .with_system(tooltip_system)
                .with_system(button_disable_system),
        )
        .init_resource::<UseSkill>()
        .add_startup_system(ui_startup_system);
    }
}
