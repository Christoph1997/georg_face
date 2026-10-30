before cargo run/build you have to source:
source /opt/ros/jazzy/setup.sh

you'll also need to source when starting the compiled executable.

ros2 topic pub /topic std_msgs/msg/String "data: 'Hello, world'"

