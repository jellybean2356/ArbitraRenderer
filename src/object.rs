use crate::vertex::Vertex;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ObjectGeometry {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl ObjectGeometry {
    pub fn load_from_arobj<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        
        let mut name = String::from("Unnamed");
        let mut obj_file: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "name" => {
                    if parts.len() > 1 {
                        name = parts[1..].join(" ");
                    }
                }
                "obj_file:" => {
                    if parts.len() > 1 {
                        obj_file = Some(parts[1].to_string());
                    }
                }
                _ => {}
            }
        }

        let obj_file = obj_file.ok_or("Missing obj_file in .arobj metadata")?;
        let obj_path = Path::new("assets").join(obj_file);

        let (models, _materials) = tobj::load_obj(
            &obj_path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        )?;

        if models.is_empty() {
            return Err("OBJ file contains no models".into());
        }

        let mesh = &models[0].mesh;
        let mut vertices = Vec::new();
        let vertex_count = mesh.positions.len() / 3;

        for i in 0..vertex_count {
            let position = [
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            ];

            let color = [1.0, 1.0, 1.0];

            let normal = if mesh.normals.is_empty() {
                [0.0, 1.0, 0.0]
            } else {
                [
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                ]
            };

            let uv = if mesh.texcoords.is_empty() {
                [0.0, 0.0]
            } else {
                [
                    mesh.texcoords[i * 2],
                    mesh.texcoords[i * 2 + 1],
                ]
            };

            vertices.push(Vertex {
                position,
                color,
                normal,
                uv,
            });
        }

        let indices: Vec<u16> = mesh.indices.iter().map(|&i| i as u16).collect();

        println!("Loaded '{}': {} vertices, {} indices", name, vertices.len(), indices.len());

        Ok(ObjectGeometry {
            name,
            vertices,
            indices,
        })
    }
}
