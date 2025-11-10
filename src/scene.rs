use crate::object::ObjectGeometry;
use crate::transform::Transform;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// global directional light (like the sun)
#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub ambient_strength: f32,
}

impl Default for Light {
    fn default() -> Self {
        Light {
            direction: [0.3, -1.0, 0.5],  // down and to the right
            color: [1.0, 1.0, 1.0],       // white
            intensity: 1.0,
            ambient_strength: 0.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectInstance {
    #[allow(dead_code)]
    pub name: String,
    pub geometry_name: String,
    pub transform: Transform,
    pub emissive: f32,  // glow strength (0.0 = no glow, >0 = glows)
}

#[derive(Debug)]
pub struct Scene {
    pub name: String,
    pub instances: Vec<ObjectInstance>,
    pub geometries: HashMap<String, ObjectGeometry>,
    pub light: Light,
}

impl Scene {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Scene {
            name,
            instances: Vec::new(),
            geometries: HashMap::new(),
            light: Light::default(),
        }
    }

    // load scene from .arsc file format
    pub fn load_from_arsc<P: AsRef<Path>>(
        path: P,
        assets_root: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path.as_ref())?;
        let mut scene = Scene::new(String::from("Unnamed Scene"));
        
        // (geometry_path, instance_name, position, rotation, scale, emissive)
        let mut current_object: Option<(String, String, [f32; 3], [f32; 3], [f32; 3], f32)> = None;

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
                "light" => {
                    // parse light definition (stays in light block until next keyword)
                }
                "light_direction:" => {
                    if parts.len() >= 4 {
                        scene.light.direction = [
                            parts[1].parse()?,
                            parts[2].parse()?,
                            parts[3].parse()?,
                        ];
                    }
                }
                "light_color:" => {
                    if parts.len() >= 4 {
                        scene.light.color = [
                            parts[1].parse()?,
                            parts[2].parse()?,
                            parts[3].parse()?,
                        ];
                    }
                }
                "light_intensity:" => {
                    if parts.len() >= 2 {
                        scene.light.intensity = parts[1].parse()?;
                    }
                }
                "ambient_strength:" => {
                    if parts.len() >= 2 {
                        scene.light.ambient_strength = parts[1].parse()?;
                    }
                }
                "object" => {
                    // finalize previous object before starting new one
                    if let Some((geom_path, inst_name, pos, rot, scl, emis)) = current_object.take() {
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
                            emissive: emis,
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
                        0.0,  // emissive default
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
                "emissive:" => {
                    if let Some(ref mut obj) = current_object {
                        if parts.len() >= 2 {
                            obj.5 = parts[1].parse()?;
                        }
                    }
                }
                _ => {}
            }
        }
        
        if let Some((geom_path, inst_name, pos, rot, scl, emis)) = current_object.take() {
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
                emissive: emis,
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
