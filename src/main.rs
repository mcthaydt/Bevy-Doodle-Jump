use bevy::{
    prelude::*,
    window::{close_on_esc, PresentMode},
};
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

// Game Constants
const WINDOW_WIDTH: i16 = 960;
const WINDOW_HEIGHT: i16 = 540;
const WINDOW_TITLE: &str = "Doodle Jump";

const SPRITE_SIZE: f32 = 50.0;
const PLATFORM_WIDTH: f32 = 30.0;
const PLATFORM_HEIGHT: f32 = 20.0;

const BACKGROUND_COLOR: &str = "7FBDF0";
const PLAYER_COLOR: &str = "2A75BE";
const PLATFORM_COLOR: &str = "094A6D";

// Components
#[derive(Component)]
struct Player {
    movement_speed: f32,
    jump_force: f32,
    player_grounded: bool,
}
#[derive(Component)]
struct PlayerCamera;
#[derive(Component)]
struct Platform;

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
        .insert_resource(ClearColor(Color::hex(BACKGROUND_COLOR).unwrap()))
        // Plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(250.0))
        // .add_plugin(RapierDebugRenderPlugin::default())
        // Startup Systems
        .add_startup_system(spawn_world_system)
        // Staged Systems
        .add_system(player_movement_system)
        .add_system(camera_follow_system)
        .add_system_to_stage(CoreStage::PostUpdate, player_ground_detection_system)
        .add_system(close_on_esc)
        // Run
        .run();
}
fn spawn_world_system(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    // Init. World Settings
    rapier_config.gravity = Vec2::new(0.0, -150.0);

    // Spawn Camera
    commands
        .spawn()
        .insert_bundle(Camera2dBundle::default())
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 0.0, 1.0,
        )))
        .insert(PlayerCamera);

    // Spawn Player
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::hex(PLAYER_COLOR).unwrap(),
                custom_size: Some(Vec2::new(SPRITE_SIZE, SPRITE_SIZE)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::zero())
        .insert(Collider::ball(SPRITE_SIZE / 2.2))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Player {
            movement_speed: 300.0,
            jump_force: 150.0,
            player_grounded: false,
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
        .insert(Platform);
}
fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<((&mut Player, &mut Velocity), With<Player>)>,
) {
    // Query Player
    let (mut player, _player_velocity) = player_query.single_mut();

    // Get Input
    let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);
    let x_input = -(left as i8) + right as i8;

    // Normalize Input
    let mut player_input_dir = Vec2::new(x_input as f32, 0.0);
    if player_input_dir != Vec2::ZERO {
        player_input_dir /= player_input_dir.length();
    }

    // Apply Force
    player.1.linvel.x = player_input_dir.x * player.0.movement_speed;
    if player.0.player_grounded == true {
        player.1.linvel.y = player.0.jump_force;
    }
}

fn camera_follow_system(
    player_query: Query<((&Transform, &Player), With<Player>)>,
    mut camera_query: Query<(&mut Transform, &PlayerCamera), (With<PlayerCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let (player_transform, _player_object) = player_query.single();
    let (mut camera_transform, _camera_object) = camera_query.single_mut();

    let follow_pos: Vec3 = Vec3::new(0.0, player_transform.0.translation.y, 1.0);
    camera_transform.translation = camera_transform
        .translation
        .lerp(follow_pos, time.delta_seconds() * 5.0);
}

fn player_ground_detection_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut player_query: Query<((Entity, &mut Player), With<Player>)>,
    platform_query: Query<(Entity, &Platform), With<Platform>>,
) {
    let (mut player_entity, _player_object) = player_query.single_mut();

    // We should only check collision type if we're already colliding
    for collision_event in collision_events.iter() {
        for (platform_entity, _platform_object) in platform_query.iter() {
            if *collision_event
                == CollisionEvent::Started(
                    player_entity.0,
                    platform_entity,
                    CollisionEventFlags::from_bits(0).unwrap(),
                )
            {
                player_entity.1.player_grounded = true;
            } else if *collision_event
                == CollisionEvent::Stopped(
                    player_entity.0,
                    platform_entity,
                    CollisionEventFlags::from_bits(0).unwrap(),
                )
            {
                player_entity.1.player_grounded = false;
            }
        }
    }
}
