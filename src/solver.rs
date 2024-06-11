use std::collections::HashMap;
use wgpu::{BindGroupLayout, BindGroupLayoutEntry, ComputePipeline};
use crate::buffer::{BufferLayout, BufferMode};
use crate::context::GpuContext;
use crate::solution::Solution;

pub struct Solver {
    pub(super) pipeline: ComputePipeline,
    pub(super) bind_group_layout: BindGroupLayout,
    pub(super) buffer_layouts: Vec<BufferLayout>,
}

pub struct SolverBuilder {
    entry_point: String,
    shader_path: String,
    buffer_layouts: Vec<BufferLayout>,
}

impl SolverBuilder {
    pub fn new() -> Self {
        Self {
            buffer_layouts: Vec::with_capacity(4),
            shader_path: String::new(),
            entry_point: String::new(),
        }
    }

    pub fn add_storage(mut self, mode: BufferMode) -> Self {
        self.buffer_layouts.push(BufferLayout { buffer_type: crate::buffer::BufferType::Storage, mode });
        self
    }

    pub fn add_uniform(mut self) -> Self {
        self.buffer_layouts.push(BufferLayout { buffer_type: crate::buffer::BufferType::Uniform, mode: BufferMode::ReadWrite });
        self
    }

    pub fn set_shader(mut self, shader_path: &str, entry_point: &str) -> Self {
        self.shader_path.push_str(shader_path);
        self.entry_point.push_str(entry_point);
        self
    }

    pub fn build(self, context: &GpuContext) -> Solver {
        let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(self.shader_path.into()),
        });

        let entries: Vec<BindGroupLayoutEntry> = self.buffer_layouts.iter().enumerate().map(
            |(i, el)|
                BindGroupLayoutEntry {
                    binding: i as u32,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: el.binding_type(),
                    count: None,
                }
        ).collect();

        let bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        });

        let compute_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = context.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: self.entry_point.as_str(),
            compilation_options: Default::default(),
        });

        Solver { pipeline, bind_group_layout, buffer_layouts: self.buffer_layouts }
    }
}

impl Solver {
    pub fn solution(&self) -> Solution {
        Solution { solver: self, buffer_holders: HashMap::with_capacity(self.buffer_layouts.len()) }
    }
}