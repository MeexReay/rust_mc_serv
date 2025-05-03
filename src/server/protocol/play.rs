use std::sync::Arc;

use crate::server::{
    ServerError, data::text_component::TextComponent, player::context::ClientContext,
};

// Отдельная функция для работы с самой игрой
pub fn handle_play_state(
    client: Arc<ClientContext>, // Контекст клиента
) -> Result<(), ServerError> {
    // Отключение игрока с сообщением
    client.protocol_helper().disconnect(TextComponent::rainbow(
        "server is in developement suka".to_string(),
    ))?;

    // TODO: Сделать отправку пакетов Play

    Ok(())
}
