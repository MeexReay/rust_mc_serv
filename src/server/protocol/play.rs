use std::{io::Cursor, sync::Arc, thread};

use rust_mc_proto::{DataWriter, Packet, read_packet};

use crate::server::{data::text_component::TextComponent, player::context::ClientContext, ServerError};

use super::id::*;

pub fn send_update_tags(client: Arc<ClientContext>) -> Result<(), ServerError> {
    // TODO: rewrite this hardcode bullshit

    client.write_packet(&Packet::from_bytes(
        clientbound::configuration::UPDATE_TAGS,
        include_bytes!("update-tags.bin"),
    ))?;

    Ok(())
}

pub fn send_registry_data(client: Arc<ClientContext>) -> Result<(), ServerError> {
    // TODO: rewrite this hardcode bullshit

    let mut registry_data = Cursor::new(include_bytes!("registry-data.bin"));

    while let Ok(mut packet) = read_packet(&mut registry_data, None) {
        packet.set_id(clientbound::configuration::REGISTRY_DATA);
        client.write_packet(&packet)?;
    }

    Ok(())
}

// Добавки в Configuration стейт чтобы все работало
pub fn handle_configuration_state(
    client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
    let mut packet = Packet::empty(clientbound::configuration::FEATURE_FLAGS);
    packet.write_varint(1)?;
    packet.write_string("minecraft:vanilla")?;
    client.write_packet(&packet)?;

    let mut packet = Packet::empty(clientbound::configuration::KNOWN_PACKS);
    packet.write_varint(1)?;
    packet.write_string("minecraft")?;
    packet.write_string("core")?;
    packet.write_string("1.21.5")?;
    client.write_packet(&packet)?;

    client.read_packet(serverbound::configuration::KNOWN_PACKS)?;

    send_registry_data(client.clone())?;
    send_update_tags(client.clone())?;

    Ok(())
}

// Отдельная функция для работы с самой игрой
pub fn handle_play_state(
    client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {

    // Отправка пакета Login
    let mut packet = Packet::empty(clientbound::play::LOGIN);

    packet.write_int(0)?; // Entity ID
    packet.write_boolean(false)?; // Is hardcore
    packet.write_varint(4)?; // Dimension Names
    packet.write_string("minecraft:overworld")?;
    packet.write_string("minecraft:nether")?;
    packet.write_string("minecraft:the_end")?;
    packet.write_string("minecraft:overworld_caves")?;
    packet.write_varint(0)?; // Max Players
    packet.write_varint(8)?; // View Distance
    packet.write_varint(5)?; // Simulation Distance
    packet.write_boolean(false)?; // Reduced Debug Info
    packet.write_boolean(true)?; // Enable respawn screen
    packet.write_boolean(false)?; // Do limited crafting

    packet.write_varint(0)?; // Dimension Type
    packet.write_string("minecraft:overworld")?; // Dimension Name
    packet.write_long(0x0f38f26ad09c3e20)?; // Hashed seed
    packet.write_byte(0)?; // Game mode
    packet.write_signed_byte(-1)?; // Previous Game mode
    packet.write_boolean(false)?; // Is Debug
    packet.write_boolean(true)?; // Is Flat
    packet.write_boolean(false)?; // Has death location
    packet.write_varint(20)?; // Portal cooldown
    packet.write_varint(60)?; // Sea level

    packet.write_boolean(false)?; // Enforces Secure Chat

    client.write_packet(&packet)?;

    thread::spawn({
        let client = client.clone();

        move || {
            let _ = client.clone().run_read_loop({
                let client = client.clone();

                move |packet| {
                    // TODO: Сделать базовые приколы типа keep-alive и другое

                    Ok(())
                }
            });
            client.close();
        }
    });

    // Отключение игрока с сообщением
    client.protocol_helper().disconnect(TextComponent::rainbow(
        "server is in developement suka".to_string(),
    ))?;

    // loop {}

    // TODO: Сделать отправку чанков

    Ok(())
}
