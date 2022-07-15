use bevy::prelude::*;
use bevy::render::camera::*;
use bevy::window::WindowResized;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ScreenUnits {
            width: 20.0,
            height: 15.0,
        })
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_sample_object)
        .add_startup_system(spawn_letterboxes)
        .add_system(change_camera_scaling)
        .add_system(move_sample_object)
        .run();
}

#[derive(Component)]
struct SampleObject { direction: i32 }

// Component for identifying letterbox entities
#[derive(Component)]
struct Letterbox { id: u32 }

// Resource which defines the dimensions of the camera's view.
#[derive(Default)]
struct ScreenUnits {
    pub width: f32,
    pub height: f32,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn spawn_sample_object(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb( 1.0, 1.0, 1.0 ),
                ..default()
            },
            transform: Transform {
                scale: Vec3::new( 1.0, 1.0, 1.0 ),
                translation:Vec3::new( 0.0, 0.0, 10.0 ), 
                ..default()
            },
            ..default()
        })
        .insert(SampleObject { direction: 1 });
}

// Causes any sample objects to bounce left and right, leaving the visible area and rendering under the letterboxes.
fn move_sample_object(
    mut objects: Query<(&mut SampleObject, &mut Transform)>,
    screen_units: Res<ScreenUnits>,
) {
    for (mut object, mut transform) in objects.iter_mut() {
        // Flip direction if far enough outside the safe area.
        if 
            transform.translation[0] > (screen_units.width + 4.0) / 2.0 || 
            transform.translation[0] < -(screen_units.width + 4.0) / 2.0 
        {
            object.direction *= -1;
        }

        // Apply movement to the transform.
        transform.translation[0] += object.direction as f32 / 6.0;
    }
}

// System to add letterboxes to the world. 
// When the window is created a window update event will be triggered automatically so we don't need to calculate their values right now.
fn spawn_letterboxes(mut commands: Commands) {
    spawn_letterbox(&mut commands, 0, Color::BLACK);
    spawn_letterbox(&mut commands, 1, Color::BLACK);
}

fn spawn_letterbox(
    commands: &mut Commands,
    id: u32,
    color: Color
) {
    commands
        .spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: color,
            ..default()
        },
        transform: Transform {
            scale: Vec3::new( 0.0, 0.0, 999.0 ),
            ..default()
        },
        ..default()
    })
    .insert(Letterbox { id: id });
}

fn change_camera_scaling(
    mut orthographic_projection_query: Query<&mut OrthographicProjection>,
    mut resize_events: EventReader<WindowResized>,
    mut letterbox_query: Query<(&Letterbox, &mut Transform)>,
    game_screen_units: Res<ScreenUnits>,
) {
    // Iterate through window resize events to find the primary window.
    for window in resize_events.iter() {
        if window.id.is_primary() == true {
            // Initialize variables with default values.
			let mut new_scaling_mode = ScalingMode::FixedVertical;
			let mut new_scale = game_screen_units.height;
            
            // If the default values are not correct then reassign them.
            if window.width / window.height < game_screen_units.width / game_screen_units.height {
                new_scaling_mode = ScalingMode::FixedHorizontal;
                new_scale = game_screen_units.width;
            } 
            
            // The new scaling mode will determine how we need to update the letterboxes
            match new_scaling_mode {
                ScalingMode::FixedVertical => set_letterboxes_vertical(
                    &game_screen_units,
                    &mut letterbox_query,
                    &window,
                ),
                ScalingMode::FixedHorizontal => set_letterboxes_horizontal(
                    &game_screen_units,
                    &mut letterbox_query,
                    &window,
                ),
                _ => {},
            }

            // Change the camera's values.
            let mut orthographic_projection = orthographic_projection_query.iter_mut().next().unwrap();
            orthographic_projection.scaling_mode = new_scaling_mode;
            orthographic_projection.scale = new_scale / 2.0;
            break;
        }
    }
}

fn set_letterboxes_vertical(
    game_screen_units: &Res<ScreenUnits>,
    letterbox_query: &mut Query<(&Letterbox, &mut Transform)>,
    window: &WindowResized
) {
    // Calculate how wide the window is in units.
    let units_per_pixel = window.height / game_screen_units.height;
    let window_unit_width = window.width / units_per_pixel;
    
    // Calculate how wide the letterboxes need to be.
    let letterbox_width = (window_unit_width - game_screen_units.width) / 2.0;

    // Calculate where the letterboxes need to go.
    let letterbox_pos_x = (letterbox_width + game_screen_units.width) / 2.0;

    for (letterbox, mut transform) in letterbox_query.iter_mut() {
        if letterbox.id == 0 {
            set_letterbox(
                &mut transform, 
                letterbox_width, 
                game_screen_units.height, 
                letterbox_pos_x,
                0.0,
            );
        } else if letterbox.id == 1 {
            set_letterbox(
                &mut transform, 
                letterbox_width, 
                game_screen_units.height, 
                -letterbox_pos_x,
                0.0,
            );
        }
    }

}

fn set_letterboxes_horizontal(
    game_screen_units: &Res<ScreenUnits>,
    letterbox_query: &mut Query<(&Letterbox, &mut Transform)>,
    window: &WindowResized
) {
    // Calculate how tall the window is in units.
    let units_per_pixel = window.width / game_screen_units.width;
    let window_unit_hight = window.height / units_per_pixel;

    // Calculate how tall the letterboxes need to be.
    let letterbox_height = (window_unit_hight - game_screen_units.height) / 2.0;

    // Calculate where the letterboxes need to go.
    let letterbox_pos_y = (letterbox_height + game_screen_units.height) / 2.0;

    for (letterbox, mut transform) in letterbox_query.iter_mut() {
        if letterbox.id == 0 {
            set_letterbox(
                &mut transform, 
                game_screen_units.width,
                letterbox_height, 
                0.0,
                letterbox_pos_y,
            );
        } else if letterbox.id == 1 {
            set_letterbox(
                &mut transform, 
                game_screen_units.width,
                letterbox_height, 
                0.0,
                -letterbox_pos_y,
            );
        }
    }
}

fn set_letterbox(
    transform: &mut Transform,
    width: f32,
    height: f32,
    x_pos: f32,
    y_pos: f32,
) {
    transform.scale = Vec3::new( width, height, 1.0 );
    transform.translation = Vec3::new( x_pos, y_pos, 999.0 );
}