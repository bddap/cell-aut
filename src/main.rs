mod camera;
mod gui;
mod quad_pipeline;
mod render;
mod utils;
mod vertex;

use std::sync::Arc;

use crate::gui::user_interface;
use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    window::{close_on_esc, WindowMode},
};
use bevy_vulkano::{
    texture_from_file_bytes, BevyVulkanoWindows, VulkanoWinitConfig, VulkanoWinitPlugin,
};
use camera::OrthographicCamera;
use render::FillScreenRenderPass;
use vulkano::{format::Format, image::ImageViewAbstract};

pub const WIDTH: f32 = 512.0;
pub const HEIGHT: f32 = 512.0;
pub const CLEAR_COLOR: [f32; 4] = [0.0; 4];
pub const CAMERA_MOVE_SPEED: f32 = 200.0;

fn main() {
    App::new()
        // Must be non send resource for now...
        .insert_non_send_resource(VulkanoWinitConfig::default())
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            title: "Cellular Automata".to_string(),
            present_mode: bevy::window::PresentMode::Immediate,
            resizable: true,
            mode: WindowMode::Windowed,
            ..WindowDescriptor::default()
        })
        // Add needed (minimum) plugins
        .add_plugin(bevy::core::CorePlugin)
        .add_plugin(bevy::log::LogPlugin)
        .add_plugin(bevy::time::TimePlugin)
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::input::InputPlugin)
        // The plugin to use vulkano with bevy
        .add_plugin(VulkanoWinitPlugin)
        .add_startup_system(setup)
        .add_system(update_camera)
        .add_system(close_on_esc)
        .add_system(input_actions)
        // gui
        .add_system(user_interface)
        .add_system_to_stage(CoreStage::PostUpdate, render)
        .run();
}

/// Vulkano image to be passed as an input to an image sample in our quad pipeline
struct TreeImage(Arc<dyn ImageViewAbstract + Send + Sync + 'static>);

/// Creates our simulation and render pipelines
fn setup(mut commands: Commands, vulkano_windows: NonSend<BevyVulkanoWindows>) {
    let (primary_window_renderer, _gui) = vulkano_windows.get_primary_window_renderer().unwrap();

    // Create our render pass
    let fill_screen = FillScreenRenderPass::new(
        primary_window_renderer.graphics_queue(),
        primary_window_renderer.swapchain_format(),
    );
    commands.insert_resource(fill_screen);

    // load the tree texture
    let tree_image = texture_from_file_bytes(
        primary_window_renderer.graphics_queue(),
        include_bytes!("../assets/dalltree.png"),
        Format::R8G8B8A8_SRGB, // required to be srg
    )
    .unwrap();
    commands.insert_resource(TreeImage(tree_image));

    // Create simple orthographic camera
    let camera = OrthographicCamera::default();
    commands.insert_resource(camera);
}

/// Render the simulation
fn render(
    mut vulkano_windows: NonSendMut<BevyVulkanoWindows>,
    mut fill_screen: ResMut<FillScreenRenderPass>,
    tree_image: Res<TreeImage>,
    camera: Res<OrthographicCamera>,
) {
    let tree_image = tree_image.0.clone();

    // Access our window renderer and gui
    let (window_renderer, gui) = vulkano_windows.get_primary_window_renderer_mut().unwrap();
    // Start frame
    let before = match window_renderer.acquire() {
        Err(e) => {
            bevy::log::error!("Failed to start frame: {}", e);
            return;
        }
        Ok(f) => f,
    };
    // Access the final window image (this is the current GPU image which changes between frames (swapchain))
    let final_image = window_renderer.swapchain_image_view();
    let after_images = fill_screen.draw(
        before,
        camera.clone(),
        tree_image,
        final_image.clone(),
        CLEAR_COLOR,
    );

    // Draw GUI using egui_winit_window's GUI draw pipeline
    let after_gui = gui.draw_on_image(after_images, final_image);

    // Finish Frame
    window_renderer.present(after_gui, true);
}

/// Update camera (if window is resized)
fn update_camera(windows: Res<Windows>, mut camera: ResMut<OrthographicCamera>) {
    let window = windows.get_primary().unwrap();
    camera.update(window.width(), window.height());
}

/// Input actions for camera movement, zoom and pausing
fn input_actions(
    time: Res<Time>,
    mut camera: ResMut<OrthographicCamera>,
    keyboard_input: Res<Input<KeyCode>>,
    mut mouse_input_events: EventReader<MouseWheel>,
) {
    // Move camera with arrows and WASD
    let up = keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up);
    let down = keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down);
    let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

    let x_axis = -(right as i8) + left as i8;
    let y_axis = -(up as i8) + down as i8;

    let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);
    if move_delta != Vec2::ZERO {
        move_delta /= move_delta.length();
        camera.pos += move_delta * time.delta_seconds() * CAMERA_MOVE_SPEED;
    }

    // Zoom camera with mouse scroll
    for e in mouse_input_events.iter() {
        if e.y < 0.0 {
            camera.scale *= 1.05;
        } else {
            camera.scale *= 1.0 / 1.05;
        }
    }
}
