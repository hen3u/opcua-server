use std::path::PathBuf;
use std::sync::Arc;

use opcua::server::prelude::*;
use opcua::sync::Mutex;

use std::process::Command;

fn main() {
    opcua::console_logging::init();

    let server = Server::new(ServerConfig::load(&PathBuf::from("./server.conf")).unwrap());

    let ns = {
        let address_space = server.address_space();
        let mut address_space = address_space.write();
        address_space.register_namespace("urn:rust-server").unwrap()
    };

    // Create an OPC UA variable to hold the CPU temperature data
    let temp_node = NodeId::new(ns, "cpu_temperature");
    {
        let address_space = server.address_space();
        let mut address_space = address_space.write();

        let _ = address_space.add_variables(
            vec![Variable::new(
                &temp_node,
                "cpu_temperature",
                "cpu_temperature",
                0 as f32,
            )],
            &NodeId::objects_folder_id(),
        );
    }
    let cpu_thermal_virtual = NodeId::new(ns, "cpu_thermal-virtual-0");
    {
        let address_space = server.address_space();
        let mut address_space = address_space.write();

        let _ = address_space.add_variables(
            vec![Variable::new(
                &cpu_thermal_virtual,
                "cpu_thermal-virtual-0",
                "cpu_thermal-virtual-0",
                0 as f32,
            )],
            &NodeId::objects_folder_id(),
        );
    }

    // Update the OPC UA variable with the CPU temperature data
    {
        let address_space = server.address_space();
        let mut address_space = address_space.write();
        if let Some(ref mut v) = address_space.find_variable_mut(temp_node.clone()) {
            let getter = AttrFnGetter::new(
                move |_, _, _, _, _, _| -> Result<Option<DataValue>, StatusCode> {
                    // Get the CPU temperature using lm-sensors
                    let output = Command::new("sensors")
                        .output()
                        .expect("Failed to execute command");
                    let temp_str = String::from_utf8_lossy(&output.stdout);
                    let temp = temp_str
                        .lines()
                        .find(|line| line.contains("Core"))
                        .and_then(|line| line.split_whitespace().nth(2))
                        .and_then(|temp_str| temp_str.strip_suffix("°C"))
                        .and_then(|temp_str| temp_str.parse::<f32>().ok())
                        .unwrap_or(0.0);
                    Ok(Some(DataValue::new_now(temp)))
                },
            );
            v.set_value_getter(Arc::new(Mutex::new(getter)));
        }
        if let Some(ref mut v) = address_space.find_variable_mut(cpu_thermal_virtual.clone()) {
            let getter = AttrFnGetter::new(
                move |_, _, _, _, _, _| -> Result<Option<DataValue>, StatusCode> {
                    // Get the CPU temperature using lm-sensors
                    let output = Command::new("sensors")
                        .output()
                        .expect("Failed to execute command");
                    let temp_str = String::from_utf8_lossy(&output.stdout);
                    let temp = temp_str
                        .lines()
                        .find(|line| line.contains("temp1"))
                        .and_then(|line| line.split_whitespace().nth(1))
                        .and_then(|temp_str| temp_str.strip_suffix("°C"))
                        .and_then(|temp_str| temp_str.parse::<f32>().ok())
                        .unwrap_or(0.0);
                    Ok(Some(DataValue::new_now(temp)))
                },
            );
            v.set_value_getter(Arc::new(Mutex::new(getter)));
        }
    }

    server.run();
}
