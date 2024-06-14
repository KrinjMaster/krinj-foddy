use rand::{thread_rng, Rng};
use std::{process::exit, time::Duration};

use bevy::{
    audio::{PlaybackMode, SpatialScale, Volume},
    core::FrameCount,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::common_conditions::on_timer,
};

enum DirectionX {
    Left,
    Right,
    None,
}

enum DirectionY {
    Up,
    Down,
    None,
}

#[derive(Component)]
struct Player {
    pos: Vec3,
    is_shooting: bool,
    direction_x: DirectionX,
    direction_y: DirectionY,
}

#[derive(Component)]
struct Projectile {
    pos: Vec3,
    is_player_projectile: bool,
}

#[derive(Component)]
struct Rocket {
    pos: Vec3,
    is_shooting: bool,
}

#[derive(Component)]
struct HealthBar {
    max_health: f32,
    current_health: f32,
}

// Settings of the game
//
//
// radius of a player ship
const PLAYER_RADIUS: f32 = 70.;
// player speed
const PLAYER_SPEED: f32 = 10.;
// width of hp bar
const HP_BAR_FULL_WIDTH: f32 = 20.;
// player max hp
const PLAYER_MAX_HP: f32 = 8.;
// rocket max hp
const ROCKET_MAX_HP: f32 = 3.;
// player projectile cooldown
const PLAYER_PROJECTILE_CD: u64 = 350;
// rocket projectile cooldown
const ROCKET_PROJECTILE_CD: u64 = 3500;
// can player and rocket projectiles collide
const IS_PLAYER_ROCKET_PROJECTILES_COLLISION: bool = true;
// enabling sounds (at your own risk, cuz sound framework is still junky)
const IS_SOUNDS_ENABLED: bool = false;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rusty Invaders".into(),
                        name: Some("Rusty Invaders.app".into()),
                        resolution: (1920., 1080.).into(),
                        focused: true,
                        visible: false,
                        enabled_buttons: bevy::window::EnabledButtons {
                            maximize: false,
                            ..Default::default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                make_visible,
                keyboard_input,
                update_rocket_pos,
                update_player_pos,
                update_projectile_pos,
                player_movement,
                rocket_movement,
                projectile_movement,
                rocket_projectile_player_collision_system,
                player_projectile_rocket_collision_system,
                player_rocket_projectile_collision,
                rocket_player_collision_system,
                spawn_rocket.run_if(on_timer(Duration::from_secs(2))),
                shoot_projectile_player
                    .run_if(on_timer(Duration::from_millis(PLAYER_PROJECTILE_CD))),
                shoot_projectile_rocket
                    .run_if(on_timer(Duration::from_millis(ROCKET_PROJECTILE_CD))),
            ),
        )
        .run();
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == 10 {
        window.single_mut().visible = true;
    }
}

fn setup(
    windows: Query<&Window>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let transform_player = Transform {
        translation: Vec3::new(-window.resolution.width() / 4., 0., 0.),
        scale: Vec3::new(6.5, 6.5, 6.5),
        ..Default::default()
    };
    let transform_hp_bar = Transform {
        translation: Vec3::new(0., -10., 0.),
        scale: Vec3::new(0.15, 0.15, 0.15),
        ..Default::default()
    };
    let color_hp_bar = Color::hex("#FF0000").unwrap_or_else(|err| {
        println!("!! Error: {}", err);
        exit(1);
    });

    // camera
    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::hex("#000000").unwrap_or_else(|err| {
                println!("!! Error: {}", err);
                exit(1);
            })),
            ..Default::default()
        },
        ..Default::default()
    });

    // player
    commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("Ship.png"),
                transform: transform_player,
                ..default()
            },
            Player {
                pos: Vec3::new(-window.resolution.width() / 4., 0., 0.),
                is_shooting: false,
                direction_x: DirectionX::None,
                direction_y: DirectionY::None,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle {
                        half_size: Vec2::new(HP_BAR_FULL_WIDTH * 2., 4.),
                    })),
                    material: materials.add(color_hp_bar),
                    transform: transform_hp_bar,
                    ..default()
                },
                HealthBar {
                    max_health: PLAYER_MAX_HP,
                    current_health: PLAYER_MAX_HP,
                },
            ));
        });
    // game stats
    //
    // enemies death count
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "enemies destroyed: 0",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/Quinquefive-ALoRM.ttf"),
                font_size: 25.0,
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
    ));

    // hp
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "HP 8/8",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/Quinquefive-ALoRM.ttf"),
                font_size: 25.0,
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    ));
}

fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut player: Query<&mut Player>) {
    for mut player in player.iter_mut() {
        if keys.pressed(KeyCode::KeyW) {
            player.direction_y = DirectionY::Up;
        }
        if keys.pressed(KeyCode::KeyS) {
            player.direction_y = DirectionY::Down;
        }
        if keys.pressed(KeyCode::KeyA) {
            player.direction_x = DirectionX::Left;
        }
        if keys.pressed(KeyCode::KeyD) {
            player.direction_x = DirectionX::Right;
        }
        if keys.just_pressed(KeyCode::Space) {
            player.is_shooting = true;
        }
    }
}

fn player_movement(mut player: Query<(&mut Transform, &mut Player)>, windows: Query<&Window>) {
    let window = windows.single();
    for (mut transform, mut player_info) in player.iter_mut() {
        match player_info.direction_y {
            DirectionY::Up => {
                if transform.translation.y + PLAYER_RADIUS + 10. < window.resolution.height() / 2. {
                    transform.translation.y += PLAYER_SPEED;
                    player_info.direction_y = DirectionY::None;
                };
            }
            DirectionY::Down => {
                if transform.translation.y - PLAYER_RADIUS - 10. > -window.resolution.height() / 2.
                {
                    transform.translation.y -= PLAYER_SPEED;
                    player_info.direction_y = DirectionY::None;
                }
            }
            DirectionY::None => (),
        }

        match player_info.direction_x {
            DirectionX::Left => {
                if transform.translation.x - PLAYER_RADIUS - 10. > -window.resolution.width() / 2. {
                    transform.translation.x -= PLAYER_SPEED;
                }
                player_info.direction_x = DirectionX::None;
            }
            DirectionX::Right => {
                if transform.translation.x + PLAYER_RADIUS + 10. < 0. {
                    transform.translation.x += PLAYER_SPEED;
                }
                player_info.direction_x = DirectionX::None;
            }
            DirectionX::None => (),
        }
    }
}

fn shoot_projectile_player(
    mut players: Query<(&mut Transform, &mut Player)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (transform, mut player) in players.iter_mut() {
        if player.is_shooting {
            let shape = Mesh2dHandle(meshes.add(Rectangle {
                half_size: Vec2::new(12., 4.),
            }));

            let color = Color::hex("#ffffff").unwrap_or_else(|err| {
                println!("!! Error: {}", err);
                exit(1);
            });

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: shape,
                    material: materials.add(color),
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                    ),
                    ..default()
                },
                Projectile {
                    pos: Vec3::new(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                    ),
                    is_player_projectile: true,
                },
            ));

            if IS_SOUNDS_ENABLED {
                commands.spawn(AudioBundle {
                    source: asset_server.load("sounds/shoot_player.wav"),
                    settings: PlaybackSettings {
                        volume: Volume::new(0.5),
                        ..Default::default()
                    },
                });
            }

            player.is_shooting = false;
        }
    }
}

fn shoot_projectile_rocket(
    rockets: Query<(&Transform, &Rocket)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (transform, rocket) in rockets.iter() {
        if rocket.is_shooting {
            let shape = Mesh2dHandle(meshes.add(Rectangle {
                half_size: Vec2::new(12., 4.),
            }));

            let color = Color::hex("#ffffff").unwrap_or_else(|err| {
                println!("!! Error: {}", err);
                exit(1);
            });

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: shape,
                    material: materials.add(color),
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                    ),
                    ..default()
                },
                Projectile {
                    pos: Vec3::new(
                        transform.translation.x,
                        transform.translation.y,
                        transform.translation.z,
                    ),
                    is_player_projectile: false,
                },
            ));
        }
    }
}

fn update_rocket_pos(mut rockets: Query<(&mut Transform, &mut Rocket)>) {
    for (transform, mut rocket) in rockets.iter_mut() {
        rocket.pos = transform.translation;
    }
}

fn update_player_pos(mut player: Query<(&mut Transform, &mut Player)>) {
    for (transform, mut player) in player.iter_mut() {
        player.pos = transform.translation;
    }
}

fn update_projectile_pos(mut projectile: Query<(&mut Transform, &mut Projectile)>) {
    for (transform, mut projectile) in projectile.iter_mut() {
        projectile.pos = transform.translation;
    }
}

fn projectile_movement(
    mut projectiles: Query<(&mut Transform, &mut Projectile, Entity)>,
    windows: Query<&Window>,
    mut commands: Commands,
) {
    let window = windows.single();

    for (mut transform, projectile, entity) in projectiles.iter_mut() {
        if projectile.is_player_projectile {
            transform.translation.x += 10.;

            if transform.translation.x > window.resolution.width() / 2. {
                commands.entity(entity).despawn();
            }
        } else {
            transform.translation.x -= 5.;

            if transform.translation.x < -window.resolution.width() / 2. {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn player_projectile_rocket_collision_system(
    projectiles: Query<(&Projectile, Entity)>,
    mut rockets: Query<(&mut Rocket, Entity, &mut Children)>,
    mut hp_bars: Query<(&mut HealthBar, Entity, &mut Transform)>,
    mut commands: Commands,
) {
    for (projectile, entity_proj) in projectiles.iter() {
        for (rocket, entity_obst, children) in rockets.iter_mut() {
            if projectile.is_player_projectile {
                let distance = (rocket.pos - projectile.pos).length();

                if distance < 35. {
                    for (mut hp_bar, hp_entity, mut transform) in hp_bars.iter_mut() {
                        if children.get(0) == Some(&hp_entity) {
                            commands.entity(entity_proj).despawn();

                            hp_bar.current_health -= 1.;

                            transform.scale.x = 0.15 * hp_bar.current_health / hp_bar.max_health;

                            if hp_bar.current_health == 0. {
                                commands.entity(entity_obst).despawn_recursive();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn rocket_projectile_player_collision_system(
    projectiles: Query<(&Projectile, Entity)>,
    mut player: Query<(&mut Player, Entity, &mut Children)>,
    mut hp_bars: Query<(&mut HealthBar, Entity, &mut Transform)>,
    mut commands: Commands,
) {
    for (projectile, entity_proj) in projectiles.iter() {
        for (player, entity_pl, children) in player.iter_mut() {
            if projectile.is_player_projectile == false {
                let distance = (player.pos - projectile.pos).length();

                if distance < 35. {
                    for (mut hp_bar, hp_entity, mut transform) in hp_bars.iter_mut() {
                        if children.get(0) == Some(&hp_entity) {
                            commands.entity(entity_proj).despawn();

                            hp_bar.current_health -= 1.;

                            transform.scale.x = 0.15 * hp_bar.current_health / hp_bar.max_health;

                            if hp_bar.current_health == 0. {
                                commands.entity(entity_pl).despawn_recursive();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn spawn_rocket(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = thread_rng();
    let window = windows.single();

    let min: f32 = -0.75;
    let max: f32 = 0.75;

    let rocket_pos_y = rng.gen_range(min..max) * window.resolution.height() / 2.;
    let is_shooting = rng.gen_range(0..5) == 3;

    let color_hp_bar = Color::hex("#FF0000").unwrap_or_else(|err| {
        println!("!! Error: {}", err);
        exit(1);
    });
    let transform_rocket = Transform {
        translation: Vec3::new(window.resolution.width() / 2. + 100., rocket_pos_y, 0.),
        scale: Vec3::new(6.5, 6.5, 6.5),
        ..Default::default()
    };
    let transform_hp_bar = Transform {
        translation: Vec3::new(0., -10., 0.),
        scale: Vec3::new(0.15, 0.15, 0.15),
        ..Default::default()
    };

    commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load("Rocket.png"),
                transform: transform_rocket,
                ..default()
            },
            Rocket {
                pos: Vec3::new(window.resolution.width() / 4., 0., 0.),
                is_shooting,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle {
                        half_size: Vec2::new(HP_BAR_FULL_WIDTH, 4.),
                    })),
                    material: materials.add(color_hp_bar),
                    transform: transform_hp_bar,
                    ..default()
                },
                HealthBar {
                    max_health: ROCKET_MAX_HP,
                    current_health: ROCKET_MAX_HP,
                },
            ));
        });
}

fn rocket_movement(
    mut rockets: Query<(&mut Transform, &mut Rocket, Entity)>,
    mut commands: Commands,
    windows: Query<&Window>,
) {
    let window = windows.single();

    for (mut transform, _, entity) in rockets.iter_mut() {
        transform.translation.x -= 3.5;

        if transform.translation.x < -window.resolution.width() / 2. {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn rocket_player_collision_system(
    rockets: Query<(&mut Rocket, Entity)>,
    mut commands: Commands,
    mut player: Query<(&mut Player, Entity, &mut Children)>,
    mut hp_bars: Query<(&mut HealthBar, Entity, &mut Transform)>,
) {
    for (player, entity_pl, children) in player.iter_mut() {
        for (rocket, entity_roc) in rockets.iter() {
            let distance = (rocket.pos - player.pos).length();

            if distance < 100. {
                for (mut hp_bar, hp_entity, mut transform) in hp_bars.iter_mut() {
                    if children.get(0) == Some(&hp_entity) {
                        commands.entity(entity_roc).despawn_recursive();

                        hp_bar.current_health -= 1.;

                        transform.scale.x = 0.15 * hp_bar.current_health / hp_bar.max_health;

                        if hp_bar.current_health == 0. {
                            commands.entity(entity_pl).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}

fn player_rocket_projectile_collision(
    projectiles: Query<(&Transform, &Projectile, Entity)>,
    mut commands: Commands,
) {
    if IS_PLAYER_ROCKET_PROJECTILES_COLLISION {
        let mut iter = projectiles.iter_combinations();
        while let Some([(transform1, projectile1, entity1), (transform2, projectile2, entity2)]) =
            iter.fetch_next()
        {
            if projectile1.is_player_projectile ^ projectile2.is_player_projectile == true {
                let distance = (transform1.translation - transform2.translation).length();

                if distance < 35. {
                    commands.entity(entity1).despawn();
                    commands.entity(entity2).despawn();
                }
            }
        }
    }
}
