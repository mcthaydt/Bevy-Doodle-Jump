use bevy::{
    prelude::*,
    render::texture::ImageSettings,
    window::{close_on_esc, PresentMode},
};
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};
use rand::Rng;

// Game Constants
const WINDOW_WIDTH: i16 = 960;
const WINDOW_HEIGHT: i16 = 540;
const WINDOW_TITLE: &str = "Doodle Jump";

const SPRITE_SIZE: f32 = 32.0 * 1.56;
const PLATFORM_WIDTH: f32 = 64.0 * 1.875;
const PLATFORM_HEIGHT: f32 = 32.0 * 0.625;

const BACKGROUND_COLOR: &str = "F8F0E3";
// const PLAYER_COLOR: &str = "2A75BE";
const PLATFORM_COLOR: &str = "040a27";

// Components
#[derive(Component)]
struct Player {
    movement_speed: f32,
    jump_force: f32,
    player_colliding: bool,
    facing_right: bool,
}
#[derive(Component)]
struct PlayerCamera {
    follow_speed: f32,
}
#[derive(Component)]
struct Platform {
    already_collided: bool,
}
#[derive(Component)]
struct ScoreUI;
struct ScoreValue(i8);

fn main() {
    App::new()
        // Resources
        .insert_resource(WindowDescriptor {
            title: WINDOW_TITLE.to_string(),
            width: WINDOW_WIDTH as f32,
            height: WINDOW_HEIGHT as f32,
            present_mode: PresentMode::Fifo,
            ..Default::default()
        })
        .insert_resource(Msaa::default())
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(ClearColor(Color::hex(BACKGROUND_COLOR).unwrap()))
        .insert_resource(ScoreValue(0))
        // Plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(350.0))
        // .add_plugin(RapierDebugRenderPlugin::default())
        // Startup Systems
        .add_startup_system(spawn_world_system)
        .add_startup_system(initilizate_window)
        // Staged Systems
        .add_system(player_input_system)
        .add_system(player_camera_follow_system)
        .add_system(update_score_system)
        .add_system(player_animation_system)
        .add_system_to_stage(CoreStage::PostUpdate, player_collision_detection_system)
        .add_system_to_stage(CoreStage::PostUpdate, player_screen_looping_system)
        .add_system(close_on_esc)
        // Run
        .run();
}

fn initilizate_window(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_visibility(false);
}

fn spawn_world_system(
    mut commands: Commands,
    mut rapier_config: ResMut<RapierConfiguration>,
    asset_server: Res<AssetServer>,
) {
    // Init. World Settings
    rapier_config.gravity = Vec2::new(0.0, -220.0);

    // Spawn Camera
    commands
        .spawn()
        .insert_bundle(Camera2dBundle::default())
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 0.0, 1.0,
        )))
        .insert(PlayerCamera { follow_speed: 5.0 });

    // Spawn UI Text
    let font = asset_server.load("Vogue.ttf");
    commands
        .spawn_bundle(
            TextBundle::from_section(
                "0.0".to_string(),
                TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    color: Color::hex("1b1b1b").unwrap(),
                },
            )
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(15.0),
                    left: Val::Px(25.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(ScoreUI);

    // Spawn Player
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                ..Default::default()
            },
            texture: asset_server.load("PlayerTexture.png"),
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Collider::ball(SPRITE_SIZE / 2.2))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Player {
            movement_speed: 300.0,
            jump_force: 200.0,
            player_colliding: false,
            facing_right: true,
        });

    // Spawn Initial Platform
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::hex(PLATFORM_COLOR).unwrap(),
                custom_size: Some(Vec2::new(PLATFORM_WIDTH, PLATFORM_HEIGHT)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, -(WINDOW_HEIGHT as f32) / 4.0, 0.0),
            ..Default::default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(
            PLATFORM_WIDTH / 2.0,
            PLATFORM_HEIGHT / 2.0,
        ))
        .insert(Platform {
            already_collided: false,
        });

    // Spawn Additional Platforms
    let mut rng = rand::thread_rng();
    for index in 1..50 {
        commands
            .spawn()
            .insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::hex(PLATFORM_COLOR).unwrap(),
                    custom_size: Some(Vec2::new(PLATFORM_WIDTH, PLATFORM_HEIGHT)),
                    ..default()
                },
                // texture: asset_server.load("PlatformTexture.png"),
                transform: Transform::from_xyz(
                    rng.gen_range(
                        -(WINDOW_WIDTH as f32 / 2.0 - PLATFORM_WIDTH as f32)
                            ..(WINDOW_WIDTH as f32 / 2.0 - PLATFORM_WIDTH as f32),
                    ),
                    -(WINDOW_HEIGHT as f32 / 4.0) + (WINDOW_HEIGHT as f32) / 4.2 * index as f32,
                    0.0,
                ),
                ..Default::default()
            })
            .insert(RigidBody::Fixed)
            .insert(Collider::cuboid(
                PLATFORM_WIDTH / 2.0,
                PLATFORM_HEIGHT / 2.0,
            ))
            .insert(Platform {
                already_collided: false,
            });
    }
}

fn player_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<((&mut Player, &mut Velocity, &mut Transform), With<Player>)>,
) {
    // Query Player
    let (mut player, _player_velocity) = player_query.single_mut();

    // Get Input
    let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    let x_input = -(left as i8) + right as i8;

    // Set Facing Direction for Animations
    if right {
        player.0.facing_right = true;
    }
    if left {
        player.0.facing_right = false;
    }

    // Normalize Input
    let mut player_input_dir = Vec2::new(x_input as f32, 0.0);
    if player_input_dir != Vec2::ZERO {
        player_input_dir /= player_input_dir.length();
    }

    // Apply Forces
    player.1.linvel.x = player_input_dir.x * player.0.movement_speed;
    if player.0.player_colliding == true {
        player.1.linvel.y = player.0.jump_force;
    }

    // Fast Fall
    let down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);

    if down {
        player.1.linvel.y = -player.0.jump_force * 3.0;
    }

    let respawn = keyboard_input.just_pressed(KeyCode::R);
    if respawn == true {
        player.2.translation = Vec3::splat(0.0);
    }
}

fn player_camera_follow_system(
    player_query: Query<((&Transform, &Player), With<Player>)>,
    mut camera_query: Query<(&mut Transform, &PlayerCamera), (With<PlayerCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    // Get player transform and camera transform
    // We also need camera object, but not player object
    let (player_transform, _player_object) = player_query.single();
    let (mut camera_transform, camera_object) = camera_query.single_mut();

    // We only need to follow the y-position
    let follow_pos: Vec3 = Vec3::new(0.0, player_transform.0.translation.y, 1.0);
    camera_transform.translation = camera_transform.translation.lerp(
        follow_pos,
        time.delta_seconds() * camera_object.follow_speed,
    );
}

fn player_collision_detection_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut score: ResMut<ScoreValue>,
    mut player_query: Query<((Entity, &mut Player), With<Player>)>,
    mut platform_query: Query<(Entity, &mut Platform), With<Platform>>,
) {
    // Rapier physics requires a reference to the entity itself for collsiion detection
    // We need grab the entity from the query- we don't need the player object
    let (mut player_entity, _player_object) = player_query.single_mut();

    for collision_event in collision_events.iter() {
        for (platform_entity, mut platform_object) in platform_query.iter_mut() {
            // We should only check collision type if we're already colliding
            if *collision_event
                == CollisionEvent::Started(
                    player_entity.0,
                    platform_entity,
                    CollisionEventFlags::from_bits(0).unwrap(),
                )
            {
                if !platform_object.already_collided == true {
                    score.0 += 1;
                }
                player_entity.1.player_colliding = true;
                platform_object.already_collided = true;
            } else if *collision_event
                == CollisionEvent::Stopped(
                    player_entity.0,
                    platform_entity,
                    CollisionEventFlags::from_bits(0).unwrap(),
                )
            {
                player_entity.1.player_colliding = false;
            }
        }
    }
}

fn player_screen_looping_system(
    mut player_query: Query<((&mut Transform, &Player), With<Player>)>,
) {
    // Get Looping Object
    let (mut player_transform, _player_object) = player_query.single_mut();

    // Snap Transform to the Opposite Side of Screen
    // 0 is center, so WINDOW / 2.0 is the actual edge
    // The bonus SPRITE_SIDE / 2.0 is just padding
    if player_transform.0.translation.x > WINDOW_WIDTH as f32 / 2.0 + SPRITE_SIZE / 2.0 as f32 {
        player_transform.0.translation.x = -(WINDOW_WIDTH as f32 / 2.0) + SPRITE_SIZE * 1.2;
    } else if player_transform.0.translation.x < -(WINDOW_WIDTH as f32 / 2.0) {
        player_transform.0.translation.x = WINDOW_WIDTH as f32 / 2.0 + SPRITE_SIZE / 2.0 as f32;
    }
}

fn player_animation_system(mut player_query: Query<((&mut Sprite, &Player), With<Player>)>) {
    // Get Player
    let (mut player_sprite, _player_object) = player_query.single_mut();

    // Determine if Sprite should be flipped or not
    if player_sprite.1.facing_right == true {
        player_sprite.0.flip_x = false;
    } else {
        player_sprite.0.flip_x = true;
    }
}

fn update_score_system(mut text_query: Query<&mut Text, With<ScoreUI>>, score: Res<ScoreValue>) {
    let mut text = text_query.single_mut();
    text.sections[0].value = score.0.to_string();
}
