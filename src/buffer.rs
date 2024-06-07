use wgpu::BufferUsages;
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
    pub(super) fn read_only(&self) -> bool {
        self.mode == Read
    }
}