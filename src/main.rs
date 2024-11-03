use bevy::prelude::*;
use bevy_tweening::{
    lens::TransformPositionLens, lens::TransformRotationLens, lens::TransformScaleLens, Animator,
    EaseFunction, RepeatCount, RepeatStrategy, Tracks, Tween, TweeningPlugin
};
use bevy::window::PrimaryWindow;
use rand::prelude::*;
use std::env::current_dir;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Resource)]
struct Resources {
    emojis: Vec<PathBuf>,
}

#[derive(Resource)]
struct Configuration {
    emoji_dir: PathBuf,
}

#[derive(Component)]
struct Animating;

const EMOJI_RESOURCES_DIR: &str = "resources/openmoji-618x618-color";

fn main() {
    let emoji_dir = current_dir().unwrap().join(EMOJI_RESOURCES_DIR);
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TweeningPlugin)
        .insert_resource(Configuration { emoji_dir })
        .add_systems(Startup, setup)
        .add_systems(Update, handle_key_presses)
        .add_systems(Update, despawn_after_animating)
        .run();
}

fn setup(mut commands: Commands, configuration: Res<Configuration>) {
    let emojis = fs::read_dir(configuration.emoji_dir.clone())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<PathBuf>>();
    commands.insert_resource(Resources { emojis });
    commands.spawn(Camera2dBundle::default());
}

fn scale_up_and_down() -> Animator<Transform> {
    let scale_up = TransformScaleLens {
        start: Vec3::new(1., 1., 1.),
        end: Vec3::new(3., 3., 3.),
    };

    let scale_down = TransformScaleLens {
        start: Vec3::new(3., 3., 3.),
        end: Vec3::new(0., 0., 0.),
    };

    let tween_scale_up = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(1),
        scale_up,
    );

    let tween_scale_down = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(1),
        scale_down,
    );

    return Animator::new(tween_scale_up.then(tween_scale_down));
}

fn rotate_and_hide() -> Animator<Transform> {
    let rotate = TransformRotationLens {
        start: Quat::from_rotation_z(0.),
        end: Quat::from_rotation_z(core::f32::consts::PI),
    };

    let scale_down = TransformScaleLens {
        start: Vec3::new(1., 1., 1.),
        end: Vec3::new(0., 0., 0.),
    };

    let tween_rotate = Tween::new(EaseFunction::QuadraticInOut, Duration::from_secs(1), rotate);

    let tween_scale_down = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(1),
        scale_down,
    );

    return Animator::new(Tracks::new(vec![tween_rotate, tween_scale_down]));
}

fn random_tween() -> Animator<Transform> {
    let random_choice = rand::thread_rng().gen_range(0..2);
    return match random_choice {
        0 => scale_up_and_down(),
        1 => rotate_and_hide(),
        _ => scale_up_and_down(),
    };
}

fn handle_key_presses(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    resources: Res<Resources>,
    query: Query<&Window, With<PrimaryWindow>>,
) {

    let width = query.iter().next().unwrap().width();
    let height = query.iter().next().unwrap().height();
    let key_it = keyboard_input.get_just_pressed();
    if key_it.len() > 0 {
        for _ in key_it {
            let rand_emoji_idx = rand::thread_rng().gen_range(0..resources.emojis.len());
            let texture_handle: Handle<Image> =
                asset_server.load(resources.emojis[rand_emoji_idx].clone());

            let rand_x = rand::thread_rng().gen_range(-width/2.0..width/2.0);
            let rand_y = rand::thread_rng().gen_range(-height/2.0..height/2.0);
            let rand_size = rand::thread_rng().gen_range(50.0..200.0);

            let random_animator = random_tween();

            commands.spawn((
                SpriteBundle {
                    texture: texture_handle,
                    transform: Transform {
                        translation: Vec3::new(rand_x, rand_y, 0.),
                        ..Default::default()
                    },
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(rand_size, rand_size)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                random_animator,
                Animating,
            ));
        }
    }
}



fn despawn_after_animating(mut commands: Commands, query: Query<(Entity, &Animator<Transform>), With<Animating>>) {
    for (entity, animator) in query.iter() {
        if animator.tweenable().progress() == 1.0 {
            commands.entity(entity).despawn();
        }
    }
}
