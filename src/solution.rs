use std::collections::HashMap;
use bytemuck::NoUninit;
use pollster::FutureExt;
use wgpu::{BindGroupEntry, Buffer, BufferUsages};
use wgpu::util::DeviceExt;
use crate::buffer::BufferMode;
use crate::context::GpuContext;
use crate::solver::Solver;

pub struct WorkGroup {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl WorkGroup {
    pub fn new(count: usize) -> Self {
        Self { x: count as u32, y: 1, z: 1 }
    }
}

pub(super) struct BufferHolder {
    input: Buffer,
    output: Option<Buffer>,
}

pub struct Solution<'a> {
    pub(super) solver: &'a Solver,
    pub(super) buffer_holders: HashMap<usize, BufferHolder>,
}

impl Solution<'_> {
    pub fn bind_data_buffer<A: NoUninit>(&mut self, index: usize, data: &[A], context: &GpuContext) {
        let layout = &self.solver.buffer_layouts[index];
        let contents = bytemuck::cast_slice(data);
        let size = (contents.len() * std::mem::size_of::<u8>()) as u64;
        let usage = layout.buffer_type.usage();

        let holder = match layout.mode {
            BufferMode::Read => {
                let input = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents,
                    usage,
                });
                BufferHolder { input, output: None }
            }
            BufferMode::Write => {
                panic!("use bind_size_buffer for write buffer")
            }
            BufferMode::ReadWrite => {
                let input = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents,
                    usage: usage | BufferUsages::COPY_SRC,
                });

                let output = context.device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                BufferHolder { input, output: Some(output) }
            }
        };

        self.buffer_holders.insert(index, holder);
    }

    pub fn bind_size_buffer(&mut self, index: usize, size: usize, context: &GpuContext) {
        let layout = &self.solver.buffer_layouts[index];

        if layout.mode != BufferMode::Write {
            panic!("use bind_data_buffer for read and readwrite buffer")
        }

        let usage = layout.buffer_type.usage();
        let size = size as u64;

        let input = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: usage | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let output = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let holder = BufferHolder { input, output: Some(output) };

        self.buffer_holders.insert(index, holder);
    }

    pub fn execute(&self, work_group: WorkGroup, context: &GpuContext) {
        let entries: Vec<BindGroupEntry> = self.buffer_holders.iter().map(
            |(&index, holder)|
                BindGroupEntry {
                    binding: index as u32,
                    resource: holder.input.as_entire_binding(),
                }
        ).collect();

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.solver.bind_group_layout,
            entries: &entries,
        });

        let mut encoder = context.device.create_command_encoder(&Default::default());

        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());
            cpass.set_pipeline(&self.solver.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(work_group.x, work_group.y, work_group.z);
        }

        for holder in self.buffer_holders.values() {
            if let Some(output) = &holder.output {
                encoder.copy_buffer_to_buffer(&holder.input, 0, output, 0, output.size());
            }
        }

        context.queue.submit(Some(encoder.finish()));
    }

    pub async fn read<A: bytemuck::Pod>(&self, index: usize, context: &GpuContext) -> Vec<A> {
        let holder = self.buffer_holders.get(&index).expect("Invalid buffer index");

        let output = holder.output.as_ref().expect("Buffer mode should be ReadWrite or Write");

        let buffer_slice = output.slice(..);
        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        context.device.poll(wgpu::Maintain::Wait);

        match receiver.await {
            Ok(Ok(())) => {
                let data = buffer_slice.get_mapped_range();
                let result = bytemuck::cast_slice(&data).to_vec();
                drop(data);
                output.unmap();
                result
            }
            Ok(Err(e)) => {
                eprintln!("Failed to map buffer: {:?}", e);
                Vec::new()
            }
            Err(e) => {
                eprintln!("Failed to receive map_async result: {:?}", e);
                Vec::new()
            }
        }
    }

    pub fn read_sync<A: bytemuck::Pod>(&self, index: usize, context: &GpuContext) -> Vec<A> {
        self.read(index, context).block_on()
    }

}