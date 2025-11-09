use crate::object::ObjectGeometry;
use crate::transform::Transform;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ObjectInstance {
    #[allow(dead_code)]
    pub name: String,
    pub geometry_name: String,
    pub transform: Transform,
}

#[derive(Debug)]
pub struct Scene {
    pub name: String,
    pub instances: Vec<ObjectInstance>,
    pub geometries: HashMap<String, ObjectGeometry>,
}

impl Scene {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Scene {
            name,
            instances: Vec::new(),
            geometries: HashMap::new(),
        }
    }

    pub fn load_from_arsc<P: AsRef<Path>>(
        path: P,
        assets_root: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path.as_ref())?;
        let mut scene = Scene::new(String::from("Unnamed Scene"));
        
        let mut current_object: Option<(String, String, [f32; 3], [f32; 3], [f32; 3])> = None;

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
                "scene_name" => {
                    if parts.len() > 1 {
                        scene.name = parts[1..].join(" ");
                    }
                }
                "object" => {
                    if let Some((geom_path, inst_name, pos, rot, scl)) = current_object.take() {
                        let arobj_path = format!("{}/{}", assets_root, geom_path);
                        let geometry = ObjectGeometry::load_from_arobj(&arobj_path)?;
                        let geometry_name = geometry.name.clone();
                        scene.geometries.entry(geometry_name.clone()).or_insert(geometry);
                        
                        let instance = ObjectInstance {
                            name: inst_name.clone(),
                            geometry_name: geometry_name.clone(),
                            transform: Transform {
                                position: pos,
                                rotation: rot,
                                scale: scl,
                            },
                        };
                        println!("Loaded instance '{}' referencing geometry '{}' at position {:?}", 
                            inst_name, geometry_name, pos);
                        scene.instances.push(instance);
                    }
                    current_object = Some((
                        String::new(),
                        String::new(),
                        [0.0, 0.0, 0.0],
                        [0.0, 0.0, 0.0],
                        [1.0, 1.0, 1.0],
                    ));
                }
                "geometry:" => {
                    if let Some(ref mut obj) = current_object {
                        if parts.len() > 1 {
                            obj.0 = parts[1].to_string();
                        }
                    }
                }
                "name:" => {
                    if let Some(ref mut obj) = current_object {
                        if parts.len() > 1 {
                            obj.1 = parts[1].to_string();
                        }
                    }
                }
                "position:" => {
                    if let Some(ref mut obj) = current_object {
                        if parts.len() >= 4 {
                            obj.2 = [
                                parts[1].parse()?,
                                parts[2].parse()?,
                                parts[3].parse()?,
                            ];
                        }
                    }
                }
                "rotation:" => {
                    if let Some(ref mut obj) = current_object {
                        if parts.len() >= 4 {
                            obj.3 = [
                                parts[1].parse()?,
                                parts[2].parse()?,
                                parts[3].parse()?,
                            ];
                        }
                    }
                }
                "scale:" => {
                    if let Some(ref mut obj) = current_object {
                        if parts.len() >= 4 {
                            obj.4 = [
                                parts[1].parse()?,
                                parts[2].parse()?,
                                parts[3].parse()?,
                            ];
                        }
                    }
                }
                _ => {}
            }
        }
        
        if let Some((geom_path, inst_name, pos, rot, scl)) = current_object.take() {
            let arobj_path = format!("{}/{}", assets_root, geom_path);
            let geometry = ObjectGeometry::load_from_arobj(&arobj_path)?;
            let geometry_name = geometry.name.clone();
            scene.geometries.entry(geometry_name.clone()).or_insert(geometry);
            
            let instance = ObjectInstance {
                name: inst_name.clone(),
                geometry_name: geometry_name.clone(),
                transform: Transform {
                    position: pos,
                    rotation: rot,
                    scale: scl,
                },
            };
            println!("Loaded instance '{}' referencing geometry '{}' at position {:?}", 
                inst_name, geometry_name, pos);
            scene.instances.push(instance);
        }

        Ok(scene)
    }

    #[allow(dead_code)]
    pub fn add_instance(&mut self, instance: ObjectInstance) {
        self.instances.push(instance);
    }

    #[allow(dead_code)]
    pub fn get_geometry(&self, name: &str) -> Option<&ObjectGeometry> {
        self.geometries.get(name)
    }
}
