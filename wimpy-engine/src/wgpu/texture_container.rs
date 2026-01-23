use super::{
    graphics_provider::GraphicsProvider,
};

use wgpu::{
    AddressMode, BindGroup, BindGroupLayout, Device, Extent3d, FilterMode, Origin3d, Queue, SurfaceTexture, Texture, TextureAspect, TextureUsages, TextureView
};

pub struct TextureContainer {
    size: Extent3d,
    view: TextureView,
    bind_groups: Vec<BindGroup>
}

const BIND_GROUP_SETS: [(FilterMode,AddressMode);6] = [
    (FilterMode::Nearest,AddressMode::ClampToEdge),
    (FilterMode::Nearest,AddressMode::Repeat),
    (FilterMode::Nearest,AddressMode::MirrorRepeat),
    (FilterMode::Linear,AddressMode::ClampToEdge),
    (FilterMode::Linear,AddressMode::Repeat),
    (FilterMode::Linear,AddressMode::MirrorRepeat),
];

const fn get_bind_group_index(filter_mode: FilterMode,address_mode: AddressMode) -> usize {
    match (filter_mode,address_mode) {
        (FilterMode::Nearest, AddressMode::ClampToEdge) => 0,
        (FilterMode::Nearest, AddressMode::Repeat) => 1,
        (FilterMode::Nearest, AddressMode::MirrorRepeat) => 2,

        (FilterMode::Linear, AddressMode::ClampToEdge) => 3,
        (FilterMode::Linear, AddressMode::Repeat) => 4,
        (FilterMode::Linear, AddressMode::MirrorRepeat) => 5,

        (FilterMode::Nearest, AddressMode::ClampToBorder) => 0, // Mask to ClampToEdge
        (FilterMode::Linear, AddressMode::ClampToBorder) => 3, // Mask to ClampToEdge
    }
}

fn get_bind_groups(texture_view: &TextureView,device: &Device,bind_group_layout: &BindGroupLayout) -> Vec<BindGroup> {
    let bind_groups = BIND_GROUP_SETS.iter().copied().map(|(filter,address)| {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address,
            address_mode_v: address,
            address_mode_w: address,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: filter,
            ..Default::default()
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }
            ],
            label: Some("Texture Bind Group"),
        })
    }).collect();

    return bind_groups;
}

struct TextureCreationParameters {
    size: (u32,u32),
    mutable: bool,
    with_data: bool
}

pub struct TextureDataWriteParameters<'a> {
    pub queue: &'a Queue,
    pub texture: &'a Texture,
    pub texture_size: Extent3d,
    pub aspect: TextureAspect,
    pub mip_level: u32,
    pub origin: Origin3d,
}

pub trait TextureData {
    fn write_to_queue(self,parameters: &TextureDataWriteParameters);
    fn size(&self) -> (u32,u32);
}

fn create_texture(
    graphics_provider: &GraphicsProvider,
    bind_group_layout: &BindGroupLayout,
    parameters: TextureCreationParameters,
) -> TextureContainer {

    let size = wgpu::Extent3d {
        width: parameters.size.0,
        height: parameters.size.1,
        depth_or_array_layers: 1,
    };

    let mut usage_flags = TextureUsages::TEXTURE_BINDING;

    if parameters.with_data {
        usage_flags |= TextureUsages::COPY_DST;
    }

    if parameters.mutable {
        usage_flags |= TextureUsages::RENDER_ATTACHMENT;
    }

    let device = graphics_provider.get_device();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,

        //Might want to make this sRGB. No idea what the fuck is going on behind the scenes with this
        format: wgpu::TextureFormat::Rgba8Unorm,

        usage: usage_flags,
        label: Some("Texture"),
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_groups = get_bind_groups(&view,&device,bind_group_layout);

    return TextureContainer {
        size,
        view,
        bind_groups
    };
}

impl TextureContainer {

    pub fn get_view(&self) -> &TextureView {
        return &self.view;
    }

    pub fn create_mutable(
        graphics_provider: &GraphicsProvider,
        bind_group_layout: &BindGroupLayout,
        size: (u32,u32)
    ) -> TextureContainer {
        create_texture(graphics_provider,bind_group_layout,TextureCreationParameters {
            size,
            with_data: false,
            mutable: true
        })
    }

    pub fn from_image(
        graphics_provider: &GraphicsProvider,
        bind_group_layout: &BindGroupLayout,
        texture_data: impl TextureData
    ) -> TextureContainer {
        let size = texture_data.size();

        let texture_container = create_texture(graphics_provider,bind_group_layout,TextureCreationParameters {
            size,
            with_data: true,
            mutable: false
        });

        texture_data.write_to_queue(&TextureDataWriteParameters {
            queue: graphics_provider.get_queue(),
            texture: texture_container.view.texture(),
            texture_size: texture_container.size,
            aspect: TextureAspect::All,
            mip_level: 0,
            origin: Origin3d::ZERO
        });

        return texture_container;
    }

    pub fn create_output(surface: &SurfaceTexture,size: (u32,u32)) -> TextureContainer {
        let view = surface.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );
        return TextureContainer {
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            view,
            bind_groups: Vec::with_capacity(0)
        };
    }
}

impl TextureContainer {
    pub fn size(&self) -> (u32,u32) {
        (self.size.width,self.size.height)
    }

    pub fn get_bind_group(&self,filter_mode: FilterMode,address_mode: AddressMode) -> Option<&BindGroup> {
        self.bind_groups.get(get_bind_group_index(filter_mode,address_mode))
    }
}
