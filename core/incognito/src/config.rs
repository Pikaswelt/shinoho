use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncognitoTheme {
    pub primary_color: String,
    pub accent_color: String,
    pub mascot: String,
}

impl Default for IncognitoTheme {
    fn default() -> Self {
        Self {
            primary_color: "#2d1b4e".to_string(), // Dark purple
            accent_color: "#9d4edd".to_string(),  // Neon purple
            mascot: "ninja-shino".to_string(),    // Ninja mascot look
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncognitoConfig {
    pub auto_enable_vpn: bool,
    pub disk_cache_enabled: bool,
    pub private_browsing: bool,
    pub theme: IncognitoTheme,
}

impl Default for IncognitoConfig {
    fn default() -> Self {
        Self {
            auto_enable_vpn: true,
            disk_cache_enabled: false,
            private_browsing: true,
            theme: IncognitoTheme::default(),
        }
    }
}
