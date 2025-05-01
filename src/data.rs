use std::{error::Error, fmt::Display};
use serde::{Deserialize, Serialize};

use rust_mc_proto::{DataReader, DataWriter, ProtocolError};
use serde_with::skip_serializing_none;

// Ошибки сервера
#[derive(Debug)]
pub enum ServerError {
    UnknownPacket(String),
    Protocol(ProtocolError),
    ConnectionClosed,
    SerTextComponent,
    DeTextComponent
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}

impl Error for ServerError {}

// Делаем чтобы ProtocolError мог переделываться в наш ServerError
impl From<ProtocolError> for ServerError {
    fn from(error: ProtocolError) -> ServerError {
        match error {
            // Если просто закрыто соединение, переделываем в нашу ошибку этого
            ProtocolError::ConnectionClosedError => {
                ServerError::ConnectionClosed
            },
            // Все остальное просто засовываем в обертку
            error => {
                ServerError::Protocol(error)
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[skip_serializing_none]
pub struct TextComponent {
    pub text: String,
    pub color: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub extra: Option<Vec<TextComponent>>,
}

impl TextComponent {
    pub fn builder() -> TextComponentBuilder {
        TextComponentBuilder::new()
    }

    pub fn to_string(self) -> Result<String, ServerError> {
        self.try_into()
    }

    pub fn from_string(text: String) -> Result<TextComponent, ServerError> {
        Self::try_from(text)
    }
}

pub trait WriteTextComponent {
    fn write_text_component(&mut self, component: &TextComponent) -> Result<(), ServerError>;
}

impl<T: DataWriter> WriteTextComponent for T {
    fn write_text_component(&mut self, component: &TextComponent) -> Result<(), ServerError> {
        Ok(self.write_string(TryInto::<String>::try_into(component.clone())?.as_str())?)
    }
}

pub trait ReadTextComponent {
    fn read_text_component(&mut self) -> Result<TextComponent, ServerError>;
}

impl<T: DataReader> ReadTextComponent for T {
    fn read_text_component(&mut self) -> Result<TextComponent, ServerError> {
        TextComponent::try_from(self.read_string()?)
    }
}

impl TryInto<String> for TextComponent {
    type Error = ServerError;

    fn try_into(self) -> Result<String, Self::Error> {
        serde_json::to_string(&self)
            .map_err(|_| ServerError::SerTextComponent)
    }
}

impl TryFrom<String> for TextComponent {
    type Error = ServerError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value)
            .map_err(|_| ServerError::DeTextComponent)
    }
}

impl TryFrom<&str> for TextComponent {
    type Error = ServerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(&value)
            .map_err(|_| ServerError::DeTextComponent)
    }
}

pub struct TextComponentBuilder {
    text: String,
    color: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    extra: Option<Vec<TextComponent>>,
}

impl TextComponentBuilder {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            color: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            extra: None,
        }
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }

    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = Some(italic);
        self
    }

    pub fn underlined(mut self, underlined: bool) -> Self {
        self.underlined = Some(underlined);
        self
    }

    pub fn strikethrough(mut self, strikethrough: bool) -> Self {
        self.strikethrough = Some(strikethrough);
        self
    }

    pub fn obfuscated(mut self, obfuscated: bool) -> Self {
        self.obfuscated = Some(obfuscated);
        self
    }

    pub fn extra(mut self, extra: Vec<TextComponent>) -> Self {
        self.extra = Some(extra);
        self
    }

    pub fn build(self) -> TextComponent {
        TextComponent { 
            text: self.text, 
            color: self.color, 
            bold: self.bold, 
            italic: self.italic, 
            underlined: self.underlined, 
            strikethrough: self.strikethrough, 
            obfuscated: self.obfuscated, 
            extra: self.extra
        }
    }
}