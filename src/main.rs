mod app_state;

#[cfg(target_os = "linux")]
mod ros2;

mod bevy_ui;

fn main() {
    let state = app_state::AppState::new();

    // Start ROS 2 only on Linux.
    #[cfg(target_os = "linux")]
    {
        let ros_state = state.clone();

        std::thread::spawn(move || {
            if let Err(err) = ros2::run(ros_state.clone()) {
                ros_state.push_output(format!("ROS 2 error: {err}"));
            }
        });
    }

    // Inform the user that ROS 2 is disabled on other platforms.
    #[cfg(not(target_os = "linux"))]
    {
        state.push_output(
            "ROS 2 communication is disabled on Windows.".to_string(),
        );
    }

    // Run the UI on the main thread, regardless of operating system.
    bevy_ui::run(state);
}