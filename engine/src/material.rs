use anyhow::{Context, Result};
use std::fs;

#[derive(Debug, Clone)]
pub struct Material {
    #[allow(dead_code)]
    pub name: String,
    pub albedo_texture: String,
    #[allow(dead_code)]
    pub roughness: f32,
    #[allow(dead_code)]
    pub metallic: f32,
}

impl Material {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read material file: {}", path))?;

        let mut name = String::from("Unnamed");
        let mut albedo_texture = String::from("textures/white.png");
        let mut roughness = 0.5;
        let mut metallic = 0.0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(value) = line.strip_prefix("name ") {
                name = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("albedo_texture: ") {
                albedo_texture = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("roughness: ") {
                roughness = value.trim().parse()
                    .with_context(|| format!("Invalid roughness value in {}", path))?;
            } else if let Some(value) = line.strip_prefix("metallic: ") {
                metallic = value.trim().parse()
                    .with_context(|| format!("Invalid metallic value in {}", path))?;
            }
        }

        Ok(Material {
            name,
            albedo_texture,
            roughness,
            metallic,
        })
    }

    pub fn default() -> Self {
        Material {
            name: String::from("Default"),
            albedo_texture: String::from("textures/white.png"),
            roughness: 0.5,
            metallic: 0.0,
        }
    }
}
