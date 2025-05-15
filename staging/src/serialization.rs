//! Serialization Examples
//!
//! This module demonstrates working with serde for serialization and deserialization:
//! - Basic serialization with serde
//! - Custom serialization/deserialization
//! - Working with different formats (JSON, binary, etc.)
//! - Handling complex data structures
//! - Error handling in serialization

use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use std::collections::HashMap;
use std::fmt;

/// Problem: Create a basic serializable struct
///
/// Define a simple struct with serde attributes
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    #[serde(default)]
    pub active: bool,
}

/// Problem: Work with nested structures
///
/// Create a complex data structure for serialization
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlogPost {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author: User,
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Problem: Use custom serialization for a field
///
/// Implement custom serialization for a specific type
#[derive(Debug, PartialEq)]
pub struct HexColor(pub u32);

impl Serialize for HexColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Format the color as a hex string with # prefix
        let hex = format!("#{:06x}", self.0);
        serializer.serialize_str(&hex)
    }
}

struct HexColorVisitor;

impl<'de> Visitor<'de> for HexColorVisitor {
    type Value = HexColor;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a hex color string like '#ff0000'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if !value.starts_with('#') || value.len() != 7 {
            return Err(E::custom(format!(
                "invalid hex color format: {}", value
            )));
        }

        // Parse the hex digits after the #
        let without_prefix = &value[1..];
        let color = u32::from_str_radix(without_prefix, 16)
            .map_err(|_| E::custom(format!("invalid hex value: {}", without_prefix)))?;

        Ok(HexColor(color))
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HexColorVisitor)
    }
}

/// Problem: Create a struct with skipped fields
///
/// Define a struct with fields that should be skipped during serialization
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Configuration {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "maxConnections")]
    pub max_connections: u32,
    #[serde(skip)]
    pub internal_state: u64,
}

/// Problem: Implement serialization for custom date format
///
/// Create a type that serializes dates in a specific format
#[derive(Debug, PartialEq)]
pub struct SimpleDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Serialize for SimpleDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Format as YYYY-MM-DD
        let formatted = format!("{:04}-{:02}-{:02}", self.year, self.month, self.day);
        serializer.serialize_str(&formatted)
    }
}

struct SimpleDateVisitor;

impl<'de> Visitor<'de> for SimpleDateVisitor {
    type Value = SimpleDate;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a date string in the format YYYY-MM-DD")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // Parse YYYY-MM-DD format
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() != 3 {
            return Err(E::custom(format!("invalid date format: {}", value)));
        }

        let year = parts[0].parse::<u16>()
            .map_err(|_| E::custom("invalid year"))?;
        let month = parts[1].parse::<u8>()
            .map_err(|_| E::custom("invalid month"))?;
        let day = parts[2].parse::<u8>()
            .map_err(|_| E::custom("invalid day"))?;

        // Basic validation
        if month < 1 || month > 12 {
            return Err(E::custom(format!("invalid month: {}", month)));
        }
        if day < 1 || day > 31 {
            return Err(E::custom(format!("invalid day: {}", day)));
        }

        Ok(SimpleDate { year, month, day })
    }
}

impl<'de> Deserialize<'de> for SimpleDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SimpleDateVisitor)
    }
}

/// Problem: Serialize an enum with different formats
///
/// Create an enum that has different representations
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum Message {
    Text(String),
    Number(i64),
    #[serde(rename = "boolean")]
    Flag(bool),
}

/// Problem: Implement a function to serialize objects generically
///
/// Create a helper for serializing to various formats
pub fn to_json<T>(value: &T) -> Result<String, String>
where
    T: Serialize,
{
    serde_json::to_string(value).map_err(|e| e.to_string())
}

/// Problem: Implement a function to deserialize objects generically
///
/// Create a helper for deserializing from various formats
pub fn from_json<T>(json: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Problem: Create a type with flattened fields
///
/// Use serde flattening to merge fields from a nested struct
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub postal_code: String,
    pub country: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    pub name: String,
    pub email: String,
    pub phone: String,
    #[serde(flatten)]
    pub address: Address,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            active: true,
        };

        let json = to_json(&user).unwrap();
        println!("User JSON: {}", json);

        let deserialized: User = from_json(&json).unwrap();
        assert_eq!(user, deserialized);
    }

    #[test]
    fn test_blog_post_serialization() {
        let mut metadata = HashMap::new();
        metadata.insert("views".to_string(), "42".to_string());

        let post = BlogPost {
            id: 1,
            title: "Serialization in Rust".to_string(),
            content: "Rust serialization is awesome!".to_string(),
            author: User {
                id: 1,
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                active: true,
            },
            tags: vec!["rust".to_string(), "serde".to_string()],
            metadata,
        };

        let json = to_json(&post).unwrap();
        println!("BlogPost JSON: {}", json);

        let deserialized: BlogPost = from_json(&json).unwrap();
        assert_eq!(post, deserialized);
    }

    #[test]
    fn test_hex_color_serialization() {
        let color = HexColor(0xFF5733);

        let json = serde_json::to_string(&color).unwrap();
        assert_eq!(json, "\"#ff5733\"");

        let deserialized: HexColor = serde_json::from_str(&json).unwrap();
        assert_eq!(color, deserialized);
    }

    #[test]
    fn test_configuration_serialization() {
        let config = Configuration {
            name: "MyApp".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test application".to_string()),
            max_connections: 100,
            internal_state: 12345, // This should be skipped
        };

        let json = to_json(&config).unwrap();
        println!("Configuration JSON: {}", json);

        // The internal_state field should be lost in serialization
        let deserialized: Configuration = from_json(&json).unwrap();

        // Create expected struct with default internal_state
        let mut expected = config.clone();
        expected.internal_state = 0;

        assert_eq!(deserialized, expected);
        // internal_state was skipped, so it should be the default value
        assert_eq!(deserialized.internal_state, 0);
    }

    #[test]
    fn test_simple_date_serialization() {
        let date = SimpleDate {
            year: 2023,
            month: 4,
            day: 15,
        };

        let json = serde_json::to_string(&date).unwrap();
        assert_eq!(json, "\"2023-04-15\"");

        let deserialized: SimpleDate = serde_json::from_str(&json).unwrap();
        assert_eq!(date, deserialized);
    }

    #[test]
    fn test_message_serialization() {
        let messages = vec![
            Message::Text("Hello, world!".to_string()),
            Message::Number(42),
            Message::Flag(true),
        ];

        for message in &messages {
            let json = to_json(message).unwrap();
            println!("Message JSON: {}", json);

            let deserialized: Message = from_json(&json).unwrap();
            assert_eq!(message, &deserialized);
        }
    }

    #[test]
    fn test_contact_serialization() {
        let contact = Contact {
            name: "Jane Smith".to_string(),
            email: "jane@example.com".to_string(),
            phone: "555-1234".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
                city: "Anytown".to_string(),
                postal_code: "12345".to_string(),
                country: "USA".to_string(),
            },
        };

        let json = to_json(&contact).unwrap();
        println!("Contact JSON: {}", json);

        // Verify that the address fields are flattened
        assert!(json.contains("\"street\""));
        assert!(!json.contains("\"address\""));

        let deserialized: Contact = from_json(&json).unwrap();
        assert_eq!(contact, deserialized);
    }
}
