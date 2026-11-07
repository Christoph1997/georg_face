## Requirements

This assumes Ubuntu 24.04 with ROS 2 Jazzy and Rust already installed.
In addition, the build needs `colcon` and `libclang` (for the Rust ROS bindings `r2r`):

```
sudo apt install ros-dev-tools clang libclang-dev build-essential
```

If ROS 2 is not loaded automatically in new terminals, run `source /opt/ros/jazzy/setup.bash` first
(or add that line to `~/.bashrc`). Check it: `echo $ROS_DISTRO` should print `jazzy`.

## Build

The face offers two actions. Their definitions are in the ROS package `pib_face_interfaces/` in this repo.
Build and source it first, because the Rust build (`r2r`) generates its types from it. Run these in the repo folder:

```
cd georg_face
colcon build --packages-select pib_face_interfaces
source install/setup.bash
cargo build --release
```

The first `cargo build` takes several minutes.

## Run

In every new terminal that starts the app, source the interface package first (after ROS 2 itself, see Requirements):

```
cd georg_face
source install/setup.bash
./target/release/pib_ui_rs
```

The app starts in fullscreen. Press `F11` to switch between fullscreen and a normal window,
and `Esc` on the face screen to close the app.

To send commands to the face (see below), use a second terminal and also run `source install/setup.bash` there.

The task manager needs the same `pib_face_interfaces` package to call the actions. If you prefer, move the two
files from `pib_face_interfaces/action/` into your existing pib interface package instead and change the
`use r2r::pib_face_interfaces::...` line in `src/ros2.rs` to that package's name.

## Windows

On Windows the app runs **without ROS 2** (the ROS bindings are only built on Linux), so there is no
`colcon`, `libclang` or `source install/setup.bash` step. Use it to try the face and screens with the keyboard.

Install once:

1. [Rust](https://rustup.rs) (`rustup-init.exe`). When asked, install the *Visual Studio C++ Build Tools*
   ("Desktop development with C++"), which Rust needs for linking.
2. [Git](https://git-scm.com/download/win) to clone the repo.

Build and start in PowerShell:

```
cargo build --release
.\target\release\pib_ui_rs.exe
```

The first build takes several minutes. The app starts in fullscreen (`F11` toggles, `Esc` closes it).
Control it with the keys described below: `1`-`4` for emotions, `C` for the confirm question, `H` for the text keyboard.

## Face control

The app draws the pib eyes and listens on `/face/emotion` (`std_msgs/String`).
Supported values (case-insensitive): `idle` (`neutral`), `happy`, `sleepy` (`tired`), `surprised`.
Unknown values fall back to `idle`.

```
ros2 topic pub --once /face/emotion std_msgs/msg/String "data: 'happy'"
```

Without ROS 2 (e.g. on Windows), press keys `1`-`4` in the window to switch between the emotions.

## Actions

Only one screen can be open at a time. A goal that arrives while another screen is showing is **rejected**.
Cancelling a goal closes its screen and returns to the eyes.

### `/face/confirm` (`pib_face_interfaces/action/Confirm`): Yes / No question

- **Goal:** `question`, the text to show. Empty text shows "Do you want to grab that bottle?".
- **Result:** `answer`: `0` = no, `1` = yes, `2` = timeout (30 s without an answer).
  Yes and no finish as *succeeded*, timeout as *aborted*.
- **Feedback:** `screen_visible` (true once the question is shown).

```
ros2 action send_goal /face/confirm pib_face_interfaces/action/Confirm "{question: 'Do you want to grab that bottle?'}" --feedback
```

Without ROS 2, press `C` on the face screen to open it; a physical keyboard answers with `Y` / `N`.

### `/face/ask_text` (`pib_face_interfaces/action/AskText`): on-screen QWERTY keyboard

- **Goal:** `title`, the text shown above the keyboard. Empty text shows "Please enter your favorite drink".
- **Result:** `success` and `text` (the typed text). Pressing Cancel, or 60 s without input, finishes as *aborted* with
  `success: false` and an empty `text`.
- **Feedback:** `screen_visible`.

```
ros2 action send_goal /face/ask_text pib_face_interfaces/action/AskText "{title: 'Please enter your favorite drink'}" --feedback
```

After an answer the face shows "Thank you!" and returns to the normal eyes.
Without ROS 2, press `H` on the face screen to open the keyboard; a physical keyboard also types into it (Enter = OK, Esc = Cancel).
