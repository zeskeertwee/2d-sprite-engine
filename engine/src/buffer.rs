use std::marker::PhantomData;
use std::ops::Deref;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer,
    BufferUsages, Device, IndexFormat, Queue, VertexBufferLayout,
};

pub trait GpuVertexBufferLayout {
    fn layout() -> VertexBufferLayout<'static>;
}

pub struct GpuBuffer {
    inner: Buffer,
    count: u32,
}

pub struct GpuVertexBuffer<T: GpuVertexBufferLayout> {
    inner: GpuBuffer,
    _vertex_type: PhantomData<T>,
}

pub struct GpuIndexBuffer<T: ToIndexFormat> {
    inner: GpuBuffer,
    _index_type: PhantomData<T>,
}

pub struct GpuUniformBuffer<T: Uniform + bytemuck::Pod + bytemuck::Zeroable> {
    inner: GpuBuffer,
    _uniform_type: PhantomData<T>,
}

impl GpuBuffer {
    pub fn new_with_data<T>(
        device: &Device,
        data: &[T],
        usage: BufferUsages,
        label: Option<&str>,
    ) -> Self
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(data),
            usage,
        });

        Self {
            inner: buffer,
            count: data.len() as u32,
        }
    }

    pub fn update<T>(&self, queue: &Queue, data: &[T])
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        queue.write_buffer(&self.inner, 0, bytemuck::cast_slice(data));
    }

    /// returns the amount of `T` entries that are present in the buffer
    pub fn data_count(&self) -> u32 {
        self.count
    }
}

impl Deref for GpuBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> GpuIndexBuffer<T>
where
    T: ToIndexFormat + bytemuck::Pod + bytemuck::Zeroable,
{
    pub fn new(device: &Device, data: &[T], label: Option<&str>) -> Self {
        let buffer = GpuBuffer::new_with_data(device, data, BufferUsages::INDEX, label);

        Self {
            inner: buffer,
            _index_type: PhantomData,
        }
    }

    pub fn index_format_static() -> IndexFormat {
        T::INDEX_FORMAT
    }

    pub fn index_format(&self) -> IndexFormat {
        T::INDEX_FORMAT
    }
}

impl<T> Deref for GpuIndexBuffer<T>
where
    T: ToIndexFormat + bytemuck::Pod + bytemuck::Zeroable,
{
    type Target = GpuBuffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait ToIndexFormat {
    const INDEX_FORMAT: IndexFormat;
}

impl ToIndexFormat for u16 {
    const INDEX_FORMAT: IndexFormat = IndexFormat::Uint16;
}

impl ToIndexFormat for u32 {
    const INDEX_FORMAT: IndexFormat = IndexFormat::Uint32;
}

impl<T> GpuVertexBuffer<T>
where
    T: GpuVertexBufferLayout + bytemuck::Pod + bytemuck::Zeroable,
{
    pub fn new(device: &Device, data: &[T], label: Option<&str>) -> Self {
        let buffer = GpuBuffer::new_with_data(device, data, BufferUsages::VERTEX, label);

        Self {
            inner: buffer,
            _vertex_type: PhantomData,
        }
    }

    pub fn vertex_layout() -> VertexBufferLayout<'static> {
        T::layout()
    }
}

impl<T> Deref for GpuVertexBuffer<T>
where
    T: GpuVertexBufferLayout + bytemuck::Pod + bytemuck::Zeroable,
{
    type Target = GpuBuffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait Uniform {
    fn bind_group_layout_entry() -> BindGroupLayoutEntry;
}

impl<T> GpuUniformBuffer<T>
where
    T: Uniform + bytemuck::Pod + bytemuck::Zeroable,
{
    pub fn new(device: &Device, data: &[T], label: Option<&str>) -> Self {
        let buffer = GpuBuffer::new_with_data(
            device,
            data,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            label,
        );

        Self {
            inner: buffer,
            _uniform_type: PhantomData,
        }
    }

    pub fn update(&self, queue: &Queue, data: &[T]) {
        self.inner.update(queue, data);
    }

    pub fn bind_group(&self, device: &Device, label: Option<&str>) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[T::bind_group_layout_entry()],
            label,
        })
    }

    pub fn bind_group_static(device: &Device, label: Option<&str>) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[T::bind_group_layout_entry()],
            label,
        })
    }
}

impl<T> Deref for GpuUniformBuffer<T>
where
    T: Uniform + bytemuck::Pod + bytemuck::Zeroable,
{
    type Target = GpuBuffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
