use std::sync::Arc;
use std::collections::HashMap;
use wgpu::{PrimitiveTopology, ShaderModuleDescriptor, util::DeviceExt};
use winit::window::Window;

use crate::vertex::Vertex;
use crate::camera::{Camera, CameraController, CameraUniform};
use crate::input::Input;
use crate::scene::Scene;
use crate::texture::Texture;

const MAX_POINT_LIGHTS: usize = 8;

// single point light (position + color + intensity)
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLight {
    position: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    _padding: f32,
}

// all point lights + count
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLightsUniform {
    lights: [PointLight; MAX_POINT_LIGHTS],
    count: u32,
    _padding: [f32; 3],
}

// global directional light data sent to GPU
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    direction: [f32; 3],
    _padding1: f32,
    color: [f32; 3],
    _padding2: f32,
    intensity: f32,
    ambient_strength: f32,
    _padding3: [f32; 2],
}

// per-instance model matrix + emissive strength
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    model: [[f32; 4]; 4],
    emissive: f32,
    _padding: [f32; 3],
}

// GPU buffers for a geometry (vertex buffer, index buffer, index count)
struct GeometryBuffers {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

// per-instance data (separate buffers prevent GPU write conflicts)
struct InstanceData {
    #[allow(dead_code)]
    model_buffer: wgpu::Buffer,
    model_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
}

pub struct State {
    pub window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    render_pipeline: wgpu::RenderPipeline,
    scene: Scene,
    geometry_buffers: HashMap<String, GeometryBuffers>,
    instance_data: Vec<InstanceData>,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    #[allow(dead_code)]
    light_buffer: wgpu::Buffer,
    #[allow(dead_code)]
    point_lights_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,

    #[allow(dead_code)]
    model_bind_group_layout: wgpu::BindGroupLayout,
    camera_controller: CameraController,
    pub input: Input,
    frame_count: u32,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

impl State {
    fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub async fn new(window: Arc<Window>) -> State {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into())
        });

        // load the default scene
        // use relative paths from current working directory
        let scene = Scene::load_from_arsc("assets/scenes/sample.arsc", "assets")
            .expect("Failed to load scene");

        println!("Scene loaded: '{}' with {} instances", scene.name, scene.instances.len());
        for instance in &scene.instances {
            println!("  - Instance '{}' using geometry '{}'", instance.name, instance.geometry_name);
        }

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let mut geometry_buffers = HashMap::new();
        for (geom_name, geometry) in &scene.geometries {
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Vertex Buffer", geom_name)),
                contents: bytemuck::cast_slice(&geometry.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Index Buffer", geom_name)),
                contents: bytemuck::cast_slice(&geometry.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            let num_indices = geometry.indices.len() as u32;

            geometry_buffers.insert(
                geom_name.clone(),
                GeometryBuffers {
                    vertex_buffer,
                    index_buffer,
                    num_indices,
                },
            );
        }

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: size.width as f32 / size.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -0.4,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera_buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: & [
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layour")
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group")
        });

        // create light uniform from scene
        println!("Light settings:");
        println!("  Direction: {:?}", scene.light.direction);
        println!("  Color: {:?}", scene.light.color);
        println!("  Intensity: {}", scene.light.intensity);
        println!("  Ambient: {}", scene.light.ambient_strength);
        
        let light_uniform = LightUniform {
            direction: scene.light.direction,
            _padding1: 0.0,
            color: scene.light.color,
            _padding2: 0.0,
            intensity: scene.light.intensity,
            ambient_strength: scene.light.ambient_strength,
            _padding3: [0.0; 2],
        };

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("light_bind_group_layout")
        });

        // collect point lights from emissive objects
        let mut point_lights = [PointLight {
            position: [0.0; 3],
            intensity: 0.0,
            color: [0.0; 3],
            _padding: 0.0,
        }; MAX_POINT_LIGHTS];
        
        let mut point_light_count = 0;
        for instance in &scene.instances {
            if instance.emissive > 0.0 && point_light_count < MAX_POINT_LIGHTS {
                point_lights[point_light_count] = PointLight {
                    position: instance.transform.position,
                    intensity: instance.emissive * 5.0,  // scale up for visibility
                    color: instance.emissive_color,
                    _padding: 0.0,
                };
                point_light_count += 1;
                println!("Point light {} at {:?} with color {:?}, intensity {}", 
                    point_light_count, instance.transform.position, instance.emissive_color, instance.emissive * 5.0);
            }
        }
        
        let point_lights_uniform = PointLightsUniform {
            lights: point_lights,
            count: point_light_count as u32,
            _padding: [0.0; 3],
        };
        
        let point_lights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("point_lights_buffer"),
            contents: bytemuck::cast_slice(&[point_lights_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: point_lights_buffer.as_entire_binding(),
                }
            ],
            label: Some("light_bind_group")
        });

        let model_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("model_bind_group_layout")
        });

        let mut instance_data = Vec::new();
        for instance in &scene.instances {
            let model_matrix = instance.transform.to_matrix();
            let model_matrix_array: &[f32; 16] = model_matrix.as_ref();
            
            let model_uniform = ModelUniform {
                model: [
                    [model_matrix_array[0], model_matrix_array[1], model_matrix_array[2], model_matrix_array[3]],
                    [model_matrix_array[4], model_matrix_array[5], model_matrix_array[6], model_matrix_array[7]],
                    [model_matrix_array[8], model_matrix_array[9], model_matrix_array[10], model_matrix_array[11]],
                    [model_matrix_array[12], model_matrix_array[13], model_matrix_array[14], model_matrix_array[15]],
                ],
                emissive: instance.emissive,
                _padding: [0.0; 3],
            };
            
            if instance.emissive > 0.0 {
                println!("Object '{}' has emissive: {}", instance.name, instance.emissive);
            }

            let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("model_buffer_{}", instance.name)),
                contents: bytemuck::cast_slice(&[model_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &model_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: model_buffer.as_entire_binding(),
                    }
                ],
                label: Some(&format!("model_bind_group_{}", instance.name)),
            });

            let texture = Texture::from_file(&device, &queue, &format!("assets/{}", instance.material.albedo_texture))
                .unwrap_or_else(|e| {
                    eprintln!("Failed to load texture for '{}': {}. Using white.", instance.name, e);
                    Texture::create_white_texture(&device, &queue)
                });

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    }
                ],
                label: Some(&format!("{} Texture Bind Group", instance.name)),
            });

            instance_data.push(InstanceData {
                model_buffer,
                model_bind_group,
                texture_bind_group,
            });
        }

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &model_bind_group_layout,
                    &light_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let camera_controller = CameraController::new(0.004);
        let input = Input::new();
        
        let (depth_texture, depth_texture_view) = Self::create_depth_texture(&device, size.width, size.height);
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: (&shader), entry_point: (Some("vs_main")), compilation_options: (Default::default()), buffers: (&[Vertex::desc()]) },
            primitive: wgpu::PrimitiveState {topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: None, unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false},
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // closer objects pass depth test
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState { count: (1), mask: (!0), alpha_to_coverage_enabled: (false) },
            fragment: Some(wgpu::FragmentState {module: &shader, entry_point: Some("fs_main"), compilation_options: Default::default(), targets: &[Some(wgpu::ColorTargetState {format: surface_format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL})]}),
            multiview: None,
            cache: None,
        });

        let state = State {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            render_pipeline,
            scene,
            geometry_buffers,
            instance_data,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            light_buffer,
            point_lights_buffer,
            light_bind_group,
            model_bind_group_layout,
            camera_controller,
            input,
            frame_count: 0,
            depth_texture,
            depth_texture_view,
        };

        state.configure_surface();

        state
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::Immediate,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>)
    {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.camera.aspect = new_size.width as f32 / new_size.height as f32;
        
        // recreate depth buffer for new window size
        let (depth_texture, depth_texture_view) = Self::create_depth_texture(&self.device, new_size.width, new_size.height);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
        
        self.configure_surface();
    }

    pub fn render(&mut self) {
        if self.size.width == 0 || self.size.height == 0 {
            return;
        }
        
        self.camera_controller.update_camera(&mut self.camera, &mut self.input);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        let surface_texture = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Outdated) => {
                self.configure_surface();
                return;
            }
            Err(e) => {
                panic!("failed to acquire next swapchain texture: {:?}", e);
            }
        };

        let texture_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0), // clear to far plane
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        renderpass.set_pipeline(&self.render_pipeline);
        renderpass.set_bind_group(0, &self.camera_bind_group, &[]);
        renderpass.set_bind_group(2, &self.light_bind_group, &[]);

        // render each object instance
        let mut rendered_count = 0;
        for (idx, instance) in self.scene.instances.iter().enumerate() {
            if let Some(buffers) = self.geometry_buffers.get(&instance.geometry_name) {
                if let Some(instance_data) = self.instance_data.get(idx) {
                    renderpass.set_bind_group(1, &instance_data.model_bind_group, &[]);
                    renderpass.set_bind_group(3, &instance_data.texture_bind_group, &[]);
                    renderpass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
                    renderpass.set_index_buffer(buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    renderpass.draw_indexed(0..buffers.num_indices, 0, 0..1);
                    rendered_count += 1;
                }
            }
        }
        
        if self.frame_count == 0 {
            println!("First frame: rendered {} instances out of {} total", rendered_count, self.scene.instances.len());
        }
        self.frame_count += 1;

        drop(renderpass);

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
     
        self.window.request_redraw();
    }
}
