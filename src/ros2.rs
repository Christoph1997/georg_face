use futures::{
    executor::LocalPool,
    future,
    stream::StreamExt,
    task::LocalSpawnExt,
};
use r2r::QosProfile;

use crate::app_state::AppState;

pub fn run(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let mut node = r2r::Node::create(ctx, "testnode", "")?;

    let mut sub = node.subscribe::<r2r::std_msgs::msg::String>(
        "/topic",
        QosProfile::default(),
    )?;

    let publisher = node.create_publisher::<r2r::std_msgs::msg::String>(
        "/topic2",
        QosProfile::default(),
    )?;

    let sub2 = node.subscribe_untyped(
        "/topic2",
        "std_msgs/msg/String",
        QosProfile::default(),
    )?;

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    // Subscriber: /topic -> publisher: /topic2
    spawner.spawn_local(async move {
        let mut count: u64 = 0;

        while let Some(msg) = sub.next().await {
            if count % 2 == 0 {
                let output = r2r::std_msgs::msg::String {
                    data: format!("({}): new msg: {}", count, msg.data),
                };

                if let Err(err) = publisher.publish(&output) {
                    eprintln!("Failed to publish on /topic2: {err}");
                }
            }

            count += 1;
        }
    })?;

    // Subscriber: /topic2 -> shared application state
    let output_state = state.clone();

    spawner.spawn_local(async move {
        sub2.for_each(|msg| {
            match msg {
                Ok(msg) => {
                    output_state.push_output(format!("topic2: new msg: {}", msg));
                }
                Err(err) => {
                    output_state.push_output(format!(
                        "Deserialization error on /topic2: {}",
                        err
                    ));
                }
            }

            future::ready(())
        })
        .await;
    })?;

    // ROS 2 event loop
    loop {
        node.spin_once(std::time::Duration::from_millis(10));
        pool.run_until_stalled();
    }
}