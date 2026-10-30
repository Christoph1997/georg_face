To compile and use this, Libclang has to be installed on your machine:
```sudo apt install clang libclang-dev build-essential```

before cargo run/build you have to source your terminal:
```source /opt/ros/jazzy/setup.sh```

you'll also need to source when starting the compiled executable which is why it's recommended to start the executable with the terminal using ./

This is currently the topic that gets displayed in the app:
ros2 topic pub /topic std_msgs/msg/String "data: 'Hello, world'"

