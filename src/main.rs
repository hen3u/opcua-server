#[macro_use]
extern crate log;

use std::sync::Arc;

use log::warn;
use opcua::server::address_space::{Variable, EventNotifier, MethodBuilder, ObjectBuilder};
use opcua::server::node_manager::memory::{
    simple_node_manager, InMemoryNodeManager, NamespaceMetadata, SimpleNodeManager,
    SimpleNodeManagerImpl,
};
use opcua::server::ServerBuilder;
use opcua::types::{DataValue, NodeId, ObjectId};

use std::process::Command;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // Create an OPC UA server with sample configuration and default node set

    let (server, handle) = ServerBuilder::new()
        .with_config_from("./server.conf")
        .with_node_manager(simple_node_manager(
            NamespaceMetadata {
                namespace_uri: "urn:DIYServer".to_owned(),
                ..Default::default()
            },
            "demo",
        ))
        .build()
        .unwrap();

    let node_manager = handle
        .node_managers()
        .get_of_type::<SimpleNodeManager>()
        .unwrap();
    let ns = handle.get_namespace_index("urn:DIYServer").unwrap();

    add_sensor_variables(ns, node_manager);

    // If you don't register a ctrl-c handler, the server will close without
    // informing clients.
    let handle_c = handle.clone();
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            warn!("Failed to register CTRL-C handler: {e}");
            return;
        }
        handle_c.cancel();
    });

    // Run the server. This does not ordinarily exit so you must Ctrl+C to terminate
    server.run().await.unwrap();
}

fn add_sensor_variables(ns: u16, manager: Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>) {
    // Create an OPC UA variable to hold the CPU temperature data
    let temp_node = NodeId::new(ns, "cpu_temperature");

    // Raspberry Pi 3 specific
    let cpu_thermal_virtual = NodeId::new(ns, "cpu_thermal-virtual-0");
    let piface_led7_node = NodeId::new(ns, "piface_led7");
    let piface_switch3_node = NodeId::new(ns, "piface_switch3");
    let object_id = NodeId::new(ns, "Methods");

    let address_space = manager.address_space();
    {
        let mut address_space = address_space.write();

        ObjectBuilder::new(&object_id, "Methods", "Methods")
            .event_notifier(EventNotifier::SUBSCRIBE_TO_EVENTS)
            .organized_by(ObjectId::ObjectsFolder)
            .insert(&mut *address_space);

        let trigger_node_id = NodeId::new(ns, "TriggerEvent");
        MethodBuilder::new(&trigger_node_id, "TriggerEvent", "TriggerEvent")
            .component_of(object_id.clone())
            .executable(true)
            .user_executable(true)
            .insert(&mut *address_space);
        manager.inner().add_method_callback(trigger_node_id, |_| {
            debug!("TriggerEvent method called");
            Ok(Vec::new())
        });

        let _ = address_space.add_variables(
            vec![Variable::new(
                &temp_node,
                "cpu_temperature",
                "cpu_temperature",
                0 as f32,
            )],
            &NodeId::objects_folder_id(),
        );

        let _ = address_space.add_variables(
            vec![Variable::new(
                &cpu_thermal_virtual,
                "cpu_thermal-virtual-0",
                "cpu_thermal-virtual-0",
                0 as f32,
            )],
            &NodeId::objects_folder_id(),
        );

        let piface_folder_id = NodeId::new(ns, "piface");
        address_space.add_folder(
            &piface_folder_id,
            "piface",
            "piface",
            &NodeId::objects_folder_id(),
        );

        let _ = address_space.add_variables(
            vec![
                Variable::new(
                    &piface_switch3_node,
                    "piface_switch3",
                    "piface_switch3",
                    false,
                ),
                Variable::new(&piface_led7_node, "piface_led7", "piface_led7", false),
            ],
            &piface_folder_id,
        );
    }

    {
        manager
            .inner()
            .add_read_callback(temp_node.clone(), move |_, _, _| {
                // Get the CPU temperature using lm-sensors
                let output = match Command::new("sensors").output() {
                    Ok(output) => output,
                    Err(err) => {
                        log::error!("Failed to execute 'sensors' command: {}", err);
                        return Ok(DataValue::new_now(0.0));
                    }
                };
                let temp_str = String::from_utf8_lossy(&output.stdout);
                let temp = temp_str
                    .lines()
                    .find(|line| line.contains("Core"))
                    .and_then(|line| line.split_whitespace().nth(2))
                    .and_then(|temp_str| temp_str.strip_suffix("°C"))
                    .and_then(|temp_str| temp_str.parse::<f32>().ok())
                    .unwrap_or(0.0);
                Ok(DataValue::new_now(temp))
            });

        manager
            .inner()
            .add_read_callback(cpu_thermal_virtual.clone(), move |_, _, _| {
                // Get the CPU temperature using lm-sensors
                let output = match Command::new("sensors").output() {
                    Ok(output) => output,
                    Err(err) => {
                        log::error!("Failed to execute 'sensors' command: {}", err);
                        return Ok(DataValue::new_now(0.0));
                    }
                };
                let temp_str = String::from_utf8_lossy(&output.stdout);
                let temp = temp_str
                    .lines()
                    .find(|line| line.contains("temp1"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|temp_str| temp_str.strip_suffix("°C"))
                    .and_then(|temp_str| temp_str.parse::<f32>().ok())
                    .unwrap_or(0.0);
                Ok(DataValue::new_now(temp))
            });
    }
}
