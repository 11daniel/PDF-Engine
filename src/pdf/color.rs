use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy)]
pub struct Color(u8, u8, u8);

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color(r, g, b)
    }

    pub fn r(&self) -> f64 {
        self.0 as f64 / 255.0
    }

    pub fn g(&self) -> f64 {
        self.1 as f64 / 255.0
    }

    pub fn b(&self) -> f64 {
        self.2 as f64 / 255.0
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string = format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2);
        serializer.serialize_str(&hex_string)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.trim_start_matches('#');

        if s.len() != 6 {
            return Err(serde::de::Error::custom("hex color must be 6 digits"));
        }

        let r = u8::from_str_radix(&s[0..2], 16)
            .map_err(|_| serde::de::Error::custom("invalid red component"))?;
        let g = u8::from_str_radix(&s[2..4], 16)
            .map_err(|_| serde::de::Error::custom("invalid green component"))?;
        let b = u8::from_str_radix(&s[4..6], 16)
            .map_err(|_| serde::de::Error::custom("invalid blue component"))?;

        Ok(Color(r, g, b))
    }
}
