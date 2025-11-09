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
    // load geometry from .arobj file format
    pub fn load_from_arobj<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path.as_ref())?;
        let mut name = String::from("Unnamed");
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let lines = content.lines();
        let mut parsing_vertices = false;
        let mut parsing_indices = false;
        let mut vertex_count = 0;
        let mut index_count = 0;
        let mut vertices_read = 0;
        let mut indices_read = 0;

        for line in lines {
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
                "vertices" => {
                    if parts.len() > 1 {
                        vertex_count = parts[1].parse()?;
                        parsing_vertices = true;
                        parsing_indices = false;
                        vertices_read = 0;
                    }
                }
                "indices" => {
                    if parts.len() > 1 {
                        index_count = parts[1].parse()?;
                        parsing_indices = true;
                        parsing_vertices = false;
                        indices_read = 0;
                    }
                }
                _ => {
                    if parsing_vertices && vertices_read < vertex_count {
                        // parse labeled format: "position: x y z  color: r g b"
                        let mut pos_idx = None;
                        let mut color_idx = None;
                        
                        for (i, part) in parts.iter().enumerate() {
                            if *part == "position:" {
                                pos_idx = Some(i + 1);
                            } else if *part == "color:" {
                                color_idx = Some(i + 1);
                            }
                        }
                        
                        if let (Some(pi), Some(ci)) = (pos_idx, color_idx) {
                            if ci >= pi + 3 && parts.len() >= ci + 3 {
                                let position = [
                                    parts[pi].parse()?,
                                    parts[pi + 1].parse()?,
                                    parts[pi + 2].parse()?,
                                ];
                                let color = [
                                    parts[ci].parse()?,
                                    parts[ci + 1].parse()?,
                                    parts[ci + 2].parse()?,
                                ];
                                vertices.push(Vertex { position, color });
                                vertices_read += 1;
                            }
                        }
                    } else if parsing_indices && indices_read < index_count {
                        for part in parts {
                            if let Ok(idx) = part.parse::<u16>() {
                                if indices_read < index_count {
                                    indices.push(idx);
                                    indices_read += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(ObjectGeometry {
            name,
            vertices,
            indices,
        })
    }
}
