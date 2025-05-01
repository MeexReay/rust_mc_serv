use std::{error::Error, fmt::Display};
use palette::{Hsl, IntoColor, Srgb};
use serde::{Deserialize, Serialize};

use rust_mc_proto::ProtocolError;
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
    pub fn new(text: String) -> Self {
        Self {
            text,
            color: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            extra: None
        }
    }

    pub fn rainbow(text: String) -> TextComponent {
        if text.is_empty() {
            return TextComponent::new(text);
        }

        let children = text.char_indices()
            .map(|(i, c)| {
                let hue = (i as f32) / (text.chars().count() as f32) * 360.0;
                let hsl = Hsl::new(hue, 1.0, 0.5);
                let rgb: Srgb = hsl.into_color();
                let r = (rgb.red * 255.0).round() as u8;
                let g = (rgb.green * 255.0).round() as u8;
                let b = (rgb.blue * 255.0).round() as u8;
                let mut component = TextComponent::new(c.to_string());
                component.color = Some(format!("#{:02X}{:02X}{:02X}", r, g, b));
                component
            })
            .collect::<Vec<TextComponent>>();
        
        let mut parent = children[0].clone();
        parent.extra = Some(children[1..].to_vec());
        parent
    }

    pub fn builder() -> TextComponentBuilder {
        TextComponentBuilder::new()
    }

    pub fn as_nbt(self) -> Result<Vec<u8>, ServerError> {
        fastnbt::to_bytes(&self)
            .map_err(|_| ServerError::SerTextComponent)
    }

    pub fn from_nbt(bytes: &[u8]) -> Result<TextComponent, ServerError> {
        fastnbt::from_bytes(bytes)
            .map_err(|_| ServerError::DeTextComponent)
    }

    pub fn as_json(self) -> Result<String, ServerError> {
        serde_json::to_string(&self)
            .map_err(|_| ServerError::SerTextComponent)
    }

    pub fn from_json(text: &str) -> Result<TextComponent, ServerError> {
        serde_json::from_str(text)
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