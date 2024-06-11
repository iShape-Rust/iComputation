use wgpu::{BindingType, BufferUsages};
use crate::buffer::BufferMode::Read;

pub(super) struct BufferLayout {
    pub(super) buffer_type: BufferType,
    pub(super) mode: BufferMode,
}

pub(super) enum BufferType {
    Uniform,
    Storage,
}

impl BufferType {
    pub(super) fn usage(&self) -> BufferUsages {
        match self {
            BufferType::Uniform => {
                BufferUsages::UNIFORM
            }
            BufferType::Storage => {
                BufferUsages::STORAGE
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum BufferMode {
    Read,
    Write,
    ReadWrite,
}

impl BufferLayout {
    pub(super) fn binding_type(&self) -> BindingType {
        match self.buffer_type {
            BufferType::Uniform => {
                BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                }
            }
            BufferType::Storage => {
                BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: self.mode == Read },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                }
            }
        }
    }
}