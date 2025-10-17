// in each state draw some words to the screen

use bevy::prelude::*;

use crate::states::WorldGenState;

#[derive(Component)]
struct GenPlatesUiText;

#[derive(Component)]
struct FinishedPlatesUiText;

#[derive(Component)]
struct GenContinentsUiText;

#[derive(Component)]
struct FinishedContinentsUiText;

#[derive(Component)]
struct JustChillUiText;

fn setup_gen_plates_ui(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("For gen_plates"),
        // Set the justification of the Text
        TextLayout::new_with_justify(Justify::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        GenPlatesUiText,
    ));
}

fn cleanup_gen_plates_ui(mut commands: Commands, q: Query<Entity, With<GenPlatesUiText>>) {
    let entity_id = q.single().unwrap();

    commands.entity(entity_id).despawn();
}

fn setup_finished_plates_ui(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("Press space to continue to generating continents"),
        // Set the justification of the Text
        TextLayout::new_with_justify(Justify::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        FinishedPlatesUiText,
    ));
}

fn cleanup_finished_plates_ui(
    mut commands: Commands,
    q: Query<Entity, With<FinishedPlatesUiText>>,
) {
    let entity_id = q.single().unwrap();

    commands.entity(entity_id).despawn();
}

fn setup_gen_continents_ui(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("For gen_continents"),
        // Set the justification of the Text
        TextLayout::new_with_justify(Justify::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        GenContinentsUiText,
    ));
}

fn cleanup_gen_continents_ui(mut commands: Commands, q: Query<Entity, With<GenContinentsUiText>>) {
    let entity_id = q.single().unwrap();

    commands.entity(entity_id).despawn();
}

fn setup_finished_continents_ui(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("Press space to continue to generating plate velocities"),
        // Set the justification of the Text
        TextLayout::new_with_justify(Justify::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        FinishedContinentsUiText,
    ));
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("Press R to re-generate continents"),
        // Set the justification of the Text
        TextLayout::new_with_justify(Justify::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            right: Val::Px(5.0),
            ..default()
        },
        FinishedContinentsUiText,
    ));
}

fn cleanup_finished_continents_ui(
    mut commands: Commands,
    q: Query<Entity, With<FinishedContinentsUiText>>,
) {
    for entity_id in q.iter() {
        commands.entity(entity_id).despawn();
    }
}

fn setup_gen_velocities_ui(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("For gen velocities"),
        // Set the justification of the Text
        TextLayout::new_with_justify(Justify::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        JustChillUiText,
    ));
}

fn cleanup_gen_velocities_ui(mut commands: Commands, q: Query<Entity, With<JustChillUiText>>) {
    let entity_id = q.single().unwrap();

    commands.entity(entity_id).despawn();
}

fn setup_just_chill_ui(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("For just_chill"),
        // Set the justification of the Text
        TextLayout::new_with_justify(Justify::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        JustChillUiText,
    ));
}

fn cleanup_just_chill_ui(mut commands: Commands, q: Query<Entity, With<JustChillUiText>>) {
    let entity_id = q.single().unwrap();

    commands.entity(entity_id).despawn();
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(WorldGenState::GenPlates), setup_gen_plates_ui)
            .add_systems(OnExit(WorldGenState::GenPlates), cleanup_gen_plates_ui)
            .add_systems(
                OnEnter(WorldGenState::FinishedPlates),
                setup_finished_plates_ui,
            )
            .add_systems(
                OnExit(WorldGenState::FinishedPlates),
                cleanup_finished_plates_ui,
            )
            .add_systems(
                OnEnter(WorldGenState::GenContinents),
                setup_gen_continents_ui,
            )
            .add_systems(
                OnExit(WorldGenState::GenContinents),
                cleanup_gen_continents_ui,
            )
            .add_systems(
                OnEnter(WorldGenState::FinishedContinents),
                setup_finished_continents_ui,
            )
            .add_systems(
                OnExit(WorldGenState::FinishedContinents),
                cleanup_finished_continents_ui,
            )
            .add_systems(
                OnEnter(WorldGenState::GenPlateVelocities),
                setup_gen_velocities_ui,
            )
            .add_systems(
                OnExit(WorldGenState::GenPlateVelocities),
                cleanup_gen_velocities_ui,
            )
            .add_systems(OnEnter(WorldGenState::JustChill), setup_just_chill_ui)
            .add_systems(OnExit(WorldGenState::JustChill), cleanup_just_chill_ui);
    }
}
