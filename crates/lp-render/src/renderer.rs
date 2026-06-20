//! wgpu 离屏渲染器。
//!
//! P1：无窗口，渲染到纹理 → PNG。
//! 贴图程序生成（纯色），不加载外部 PNG。

use bytemuck::{Pod, Zeroable};
use std::num::NonZeroU64;
use wgpu::util::DeviceExt;

/// 渲染目标配置。
pub struct Renderer {
    pub width: u32,
    pub height: u32,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

/// 单个 region 的渲染数据（蒙皮后）。
pub struct RegionDraw {
    /// 顶点 × (position[2] + uv[2])。矩形 region 为 4 顶点；mesh 可任意数量。
    /// 多顶点时用扇形三角化（fan：以 v0 为中心）填充。
    pub vertices: Vec<[f32; 4]>,
    /// 贴图颜色（RGBA，0~1）
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

impl Renderer {
    /// 初始化 wgpu（headless，无 Surface）。默认用高性能 GPU。
    pub fn new(width: u32, height: u32) -> Self {
        Self::new_with_options(width, height, false)
    }

    /// 初始化 wgpu，可选软件渲染。
    ///
    /// - `prefer_software=true`：用 CPU 软件适配器（Microsoft Basic Render Driver），
    ///   慢但稳定，适合测试（绕开 GPU 驱动 flaky）。
    /// - `prefer_software=false`：高性能 GPU（默认，生产用）。
    pub fn new_with_options(width: u32, height: u32, prefer_software: bool) -> Self {
        pollster::block_on(async {
            let instance = wgpu::Instance::default();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: if prefer_software {
                        wgpu::PowerPreference::LowPower
                    } else {
                        wgpu::PowerPreference::HighPerformance
                    },
                    force_fallback_adapter: prefer_software,
                    compatible_surface: None,
                })
                .await
                .expect("无法找到 wgpu 适配器（GPU 驱动？）");

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("lp-render device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::default(),
                }, None)
                .await
                .expect("无法创建 wgpu 设备");

            Self { width, height, device, queue }
        })
    }

    /// 渲染一组 region 到 PNG 文件。
    ///
    /// 坐标系：顶点 position 假设已是像素坐标（y 向上）；
    /// 内部转成 NDC（y 翻转向下，适配 wgpu 纹理原点在左上）。
    pub fn render_to_png(&self, regions: &[RegionDraw], out_path: &std::path::Path) {
        let (w, h) = (self.width as f32, self.height as f32);

        // —— 组装顶点缓冲 ——
        let mut verts: Vec<GpuVertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();
        for r in regions {
            let base = verts.len() as u16;
            let n = r.vertices.len() as u16;
            for v in &r.vertices {
                // 像素 → NDC，y 翻转
                let ndc_x = (v[0] / w) * 2.0 - 1.0;
                let ndc_y = 1.0 - (v[1] / h) * 2.0;
                verts.push(GpuVertex { position: [ndc_x, ndc_y], uv: [v[2], v[3]] });
            }
            // 扇形三角化（fan）：以 v0 为中心，生成 n-2 个三角形。
            // 4 顶点时等价原来的两个三角形；多顶点 mesh 也能正确填充。
            for i in 1..(n - 1) {
                indices.extend_from_slice(&[base, base + i, base + i + 1]);
            }
        }

        let vertex_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertices"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // —— 程序生成贴图（每个 region 一个 1×1 纯色贴图）——
        // 简化：用一个共享的白色贴图，颜色通过 uniform 传入
        let mut textures = Vec::new();
        let mut bind_groups = Vec::new();
        for r in regions {
            let tex = self.make_solid_texture([255, 255, 255, 255], 1, 1);
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor::default());
            // color uniform
            let color_buf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("color"),
                contents: bytemuck::cast_slice(&r.color),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            textures.push((tex, sampler));
            let _ = &textures; // 保持纹理存活
            bind_groups.push((color_buf, textures.len() - 1));
        }

        // —— shader ——
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { multisampled: false, sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2 },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: NonZeroU64::new(16) },
                    count: None,
                },
            ],
        });
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    // P1：纯色完全不透明，直接覆盖（不预乘 blend）
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // —— 渲染目标纹理 ——
        let target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("target"),
            size: wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

        // —— 实际绑定 bind groups（每个 region 一个）——
        let mut bg_list = Vec::new();
        for (color_buf, tex_idx) in &bind_groups {
            let (tex, sampler) = &textures[*tex_idx];
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bg"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
                    wgpu::BindGroupEntry { binding: 2, resource: color_buf.as_entire_binding() },
                ],
            });
            bg_list.push(bg);
        }

        // —— 录制命令 ——
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rpass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            rpass.set_pipeline(&pipeline);
            // 每个 region 画一次，按实际顶点/索引数累积偏移（支持多顶点 mesh）
            let mut vert_byte_off: u64 = 0;
            let mut idx_byte_off: u64 = 0;
            for (i, r) in regions.iter().enumerate() {
                let n = r.vertices.len() as u32;
                let tri_count = n.saturating_sub(2);
                let idx_count = tri_count * 3;
                rpass.set_bind_group(0, &bg_list[i], &[]);
                rpass.set_vertex_buffer(0, vertex_buf.slice(vert_byte_off..));
                rpass.set_index_buffer(index_buf.slice(idx_byte_off..), wgpu::IndexFormat::Uint16);
                rpass.draw_indexed(0..idx_count, 0, 0..1);
                vert_byte_off += (n as u64) * 16; // 每顶点 16 字节
                idx_byte_off += (idx_count as u64) * 2; // 每索引 2 字节
            }
        }

        // —— 拷贝到缓冲并读回 ——
        let bytes_per_row = self.width * 4;
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (bytes_per_row as u64) * (self.height as u64),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::ImageCopyBuffer {
                buffer: &buf,
                layout: wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(bytes_per_row), rows_per_image: Some(self.height) },
            },
            wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // —— 读回 + 写 PNG ——
        let (tx, rx) = std::sync::mpsc::channel();
        let slice = buf.slice(..);
        slice.map_async(wgpu::MapMode::Read, move |res| { tx.send(res).unwrap(); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().expect("buffer map failed");

        let data = slice.get_mapped_range().to_vec();
        let _ = slice; // BufferSlice 是 Copy，无需 drop

        // Rgba8Unorm → 写 PNG（注意 unorm 是归一化，字节序已是 RGBA）
        let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            image::ImageBuffer::from_raw(self.width, self.height, data).unwrap();
        img.save(out_path).expect("写 PNG 失败");
    }

    /// 程序生成纯色贴图。
    fn make_solid_texture(&self, color: [u8; 4], w: u32, h: u32) -> wgpu::Texture {
        let size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("solid"),
            size,
            mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let data: Vec<u8> = color.repeat((w * h) as usize);
        self.queue.write_texture(
            wgpu::ImageCopyTexture { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &data,
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h) },
            size,
        );
        tex
    }
}

const SHADER_SRC: &str = r#"
struct VOut {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_main(@location(0) position: vec2f, @location(1) uv: vec2f) -> VOut {
    var o: VOut;
    o.pos = vec4f(position, 0.0, 1.0);
    o.uv = uv;
    return o;
}

@group(0) @binding(0) var t: texture_2d<f32>;
@group(0) @binding(1) var s: sampler;
@group(0) @binding(2) var<uniform> color: vec4f;

@fragment
fn fs_main(in: VOut) -> @location(0) vec4f {
    let tex = textureSample(t, s, in.uv);
    return tex * color;
}
"#;
