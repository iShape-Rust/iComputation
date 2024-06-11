use pollster::FutureExt;
use wgpu::{Device, Queue};

pub struct GpuContext {
    pub device: Device,
    pub queue: Queue,
}

impl GpuContext {
    pub fn new_sync() -> Self {
        Self::new().block_on()
    }

    pub async fn new() -> Self {
        let instance = wgpu::Instance::default();

        let adapter = instance.request_adapter(&Default::default()).await.unwrap();
        println!("Using adapter: {:?}", adapter.get_info());
        let features = adapter.features();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        Self { device, queue }
    }
}