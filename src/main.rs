mod app_state;
mod ros2;
mod ui;

fn main() {
    let state = app_state::AppState::new();

    // Start ROS 2 independently of the UI.
    let ros_state = state.clone();

    std::thread::spawn(move || {
        if let Err(err) = ros2::run(ros_state.clone()) {
            ros_state.push_output(format!("ROS 2 error: {err}"));
        }
    });

    // Run the UI on the main thread.
    ui::run(state);
}