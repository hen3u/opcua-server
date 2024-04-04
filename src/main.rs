use std::path::PathBuf;
use std::sync::Arc;

use opcua::server::{
    address_space::method::MethodBuilder, callbacks, prelude::*, session::SessionManager,
};

use opcua::sync::Mutex;
use opcua::sync::RwLock;

use std::process::Command;

struct TriggerEvent;

impl callbacks::Method for TriggerEvent {
    fn call(
        &mut self,
        _session_id: &NodeId,
        _session_map: Arc<RwLock<SessionManager>>,
        _request: &CallMethodRequest,
    ) -> Result<CallMethodResult, StatusCode> {
        Ok(CallMethodResult {
            status_code: StatusCode::Good,
            input_argument_results: None,
            input_argument_diagnostic_infos: None,
            output_arguments: None,
        })
    }
}

fn main() {
    opcua::console_logging::init();

    let server = Server::new(ServerConfig::load(&PathBuf::from("./server.conf")).unwrap());

    let ns = {
        let address_space = server.address_space();
        let mut address_space = address_space.write();
        address_space.register_namespace("urn:rust-server").unwrap()
    };

    {
        let address_space = server.address_space();
        let mut address_space = address_space.write();

        let object_id = NodeId::new(ns, "Methods");
        ObjectBuilder::new(&object_id, "Methods", "Methods")
            .event_notifier(EventNotifier::SUBSCRIBE_TO_EVENTS)
            .organized_by(ObjectId::ObjectsFolder)
            .insert(&mut address_space);

        let fn_node_id = NodeId::new(ns, "TriggerEvent");
        MethodBuilder::new(&fn_node_id, "TriggerEvent", "TriggerEvent")
            .component_of(object_id.clone())
            .callback(Box::new(TriggerEvent))
            .insert(&mut address_space);
    }

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

    {
        let piface_led7_node = NodeId::new(ns, "piface_led7");
        let piface_switch3_node = NodeId::new(ns, "piface_switch3");

        let address_space = server.address_space();
        let mut address_space = address_space.write();

        let piface_nodeid = address_space
            .add_folder("piface", "piface", &NodeId::objects_folder_id())
            .unwrap();

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
            &piface_nodeid,
        );
    }

    // Update the OPC UA variable with the CPU temperature data
    {
        let address_space = server.address_space();
        let mut address_space = address_space.write();

        // Update the temperature variable
        if let Some(ref mut v) = address_space.find_variable_mut(temp_node.clone()) {
            let getter = AttrFnGetter::new(
                move |_, _, _, _, _, _| -> Result<Option<DataValue>, StatusCode> {
                    // Get the CPU temperature using lm-sensors
                    let output = match Command::new("sensors").output() {
                        Ok(output) => output,
                        Err(err) => {
                            log::error!("Failed to execute 'sensors' command: {}", err);
                            return Ok(Some(DataValue::new_now(0.0)));
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
                    Ok(Some(DataValue::new_now(temp)))
                },
            );
            v.set_value_getter(Arc::new(Mutex::new(getter)));
        } else {
            log::warn!("Temperature variable not found in the address space");
        }

        // Update the CPU thermal virtual variable
        if let Some(ref mut v) = address_space.find_variable_mut(cpu_thermal_virtual.clone()) {
            let getter = AttrFnGetter::new(
                move |_, _, _, _, _, _| -> Result<Option<DataValue>, StatusCode> {
                    // Get the CPU temperature using lm-sensors
                    let output = match Command::new("sensors").output() {
                        Ok(output) => output,
                        Err(err) => {
                            log::error!("Failed to execute 'sensors' command: {}", err);
                            return Ok(Some(DataValue::new_now(0.0)));
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
                    Ok(Some(DataValue::new_now(temp)))
                },
            );
            v.set_value_getter(Arc::new(Mutex::new(getter)));
        } else {
            log::warn!("CPU thermal virtual variable not found in the address space");
        }
    }

    server.run();
}
