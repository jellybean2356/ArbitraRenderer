use std::sync::Arc;
use std::collections::HashMap;
use wgpu::{PrimitiveTopology, ShaderModuleDescriptor, util::DeviceExt};
use winit::window::Window;

use crate::vertex::Vertex;
use crate::camera::{Camera, CameraController, CameraUniform};
use crate::input::Input;
use crate::scene::Scene;

/// GPU buffers for a geometry (vertex buffer, index buffer, index count)
struct GeometryBuffers {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

/// Per-instance rendering data
struct InstanceData {
    #[allow(dead_code)]
    model_buffer: wgpu::Buffer,
    model_bind_group: wgpu::BindGroup,
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
    model_bind_group_layout: wgpu::BindGroupLayout,
    camera_controller: CameraController,
    pub input: Input,
    frame_count: u32,
}

impl State {
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

        // Load the default scene
        let scene = Scene::load_from_arsc("/assets/scenes/sample.arsc", "/assets")
            .expect("Failed to load scene");

        println!("Scene loaded: '{}' with {} instances", scene.name, scene.instances.len());
        for instance in &scene.instances {
            println!("  - Instance '{}' using geometry '{}'", instance.name, instance.geometry_name);
        }

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

        let model_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
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
            label: Some("model_bind_group_layout")
        });

        let mut instance_data = Vec::new();
        for instance in &scene.instances {
            let model_matrix = instance.transform.to_matrix();
            let model_matrix_array: &[f32; 16] = model_matrix.as_ref();
            
            let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("model_buffer_{}", instance.name)),
                contents: bytemuck::cast_slice(model_matrix_array),
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

            instance_data.push(InstanceData {
                model_buffer,
                model_bind_group,
            });
        }

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &model_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let camera_controller = CameraController::new(0.004);
        let input = Input::new();
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState { module: (&shader), entry_point: (Some("vs_main")), compilation_options: (Default::default()), buffers: (&[Vertex::desc()]) },
            primitive: wgpu::PrimitiveState {topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: Some(wgpu::Face::Back), unclipped_depth: false, polygon_mode: wgpu::PolygonMode::Fill, conservative: false},
            depth_stencil: None,
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
            model_bind_group_layout,
            camera_controller,
            input,
            frame_count: 0,
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
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        renderpass.set_pipeline(&self.render_pipeline);
        renderpass.set_bind_group(0, &self.camera_bind_group, &[]);

        let mut rendered_count = 0;
        for (idx, instance) in self.scene.instances.iter().enumerate() {
            if let Some(buffers) = self.geometry_buffers.get(&instance.geometry_name) {
                if let Some(instance_data) = self.instance_data.get(idx) {
                    renderpass.set_bind_group(1, &instance_data.model_bind_group, &[]);
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
