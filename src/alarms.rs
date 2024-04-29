use opcua::server::{self, prelude::*};

pub fn add_alarms(server: &mut Server, ns: u16) {
    let address_space = server.address_space();
    let mut address_space = address_space.write();

    // Create a folder under static folder
    let devices_folder_id = address_space
        .add_folder("Alarms", "Alarms", &NodeId::objects_folder_id())
        .unwrap();

    // Instantiate alarm object
    let temperature_alarm_id = NodeId::new(ns, "TemperatureAlarm");
    ObjectBuilder::new(
        &temperature_alarm_id,
        "TemperatureAlarm",
        "TemperatureAlarm",
    )
    // .is_abstract(false)
    // .subtype_of(ObjectTypeId::BaseObjectType)
    // .generates_event(ObjectTypeId::ExclusiveLevelAlarmType)
    .organized_by(devices_folder_id)
    .has_type_definition(ObjectTypeId::ExclusiveLevelAlarmType)
    .insert(&mut address_space);

    // let a_node = NodeId::new(0, 2253);
    // let references: Option<Vec<server::address_space::references::Reference>> = address_space.find_references(&a_node, Some((ReferenceTypeId::HasComponent, false)));
    process_references(
        &address_space,
        &NodeId::new(0, 9482),
        ReferenceTypeId::HasSubtype,
        false,
    );
}

fn process_references(
    address_space: &server::address_space::AddressSpace,
    node_id: &NodeId,
    reference_type: ReferenceTypeId,
    is_inverse: bool,
) {
    let references: Option<Vec<server::address_space::references::Reference>> =
        address_space.find_inverse_references(node_id, Some((reference_type, is_inverse)));

    if let Some(references) = references {
        for reference in &references {
            log::debug!(
                "Node {} is a key in references_to_map",
                reference.target_node
            );
            println!(
                "REFS target_node {} reference_type {}",
                reference.target_node, reference.reference_type
            );

            if let Some(node) = address_space.find_node(&reference.target_node) {
                let node = node.as_node();
                println!("REFS browse_name {} ", node.browse_name().name);

                // Recursive call for the target node
                process_references(
                    address_space,
                    &reference.target_node,
                    reference_type,
                    is_inverse,
                );
            }
        }
    }
}
