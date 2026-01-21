use super::{
    graphics_provider::GraphicsProvider,
};

use image::{
    DynamicImage,
    EncodableLayout,
    GenericImageView
};

use wgpu::{
    AddressMode, BindGroup, BindGroupLayout, Device, Features, FilterMode, SurfaceTexture, TextureUsages, TextureView
};
pub struct TextureContainer {
    width: u32,
    height: u32,
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
    mutable: bool
}

fn create_texture(
    graphics_provider: &GraphicsProvider,
    bind_group_layout: &BindGroupLayout,
    image_data: Option<&[u8]>,
    parameters: TextureCreationParameters
) -> TextureContainer {

    let size = parameters.size;

    let texture_size = wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
    };

    let mut usage_flags = TextureUsages::TEXTURE_BINDING;

    if image_data.is_some() {
        usage_flags |= TextureUsages::COPY_DST;
    }

    if parameters.mutable {
        usage_flags |= TextureUsages::RENDER_ATTACHMENT;
    }

    let device = graphics_provider.get_device();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,

        //Might want to make this sRGB. No idea what the fuck is going on behind the scenes with this
        format: wgpu::TextureFormat::Rgba8Unorm,

        usage: usage_flags,
        label: Some("Texture"),
        view_formats: &[],
    });

    if let Some(data) = image_data {
        graphics_provider.get_queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                /* 1 byte per color in 8bit 4 channel color (RGBA with u8) */
                bytes_per_row: Some(4*size.0), 
                rows_per_image: Some(size.1),
            },
            texture_size,
        );
    }

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_groups = get_bind_groups(&view,&device,bind_group_layout);

    return TextureContainer {
        width: size.0,
        height: size.1,
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
        return create_texture(graphics_provider,bind_group_layout,None,TextureCreationParameters {
            size,
            mutable: true
        });
    }

    pub fn from_image(graphics_provider: &GraphicsProvider,bind_group_layout: &BindGroupLayout,image: &DynamicImage) -> TextureContainer {
        let size = image.dimensions();

        //TODO: Make sure alpha channel is premultiplied ... Somehow.
        let image_data = image.to_rgba8();

        return create_texture(graphics_provider,bind_group_layout,Some(image_data.as_bytes()),TextureCreationParameters {
            size,
            mutable: false
        });
    }

    pub fn create_output(surface: &SurfaceTexture,size: (u32,u32)) -> TextureContainer {
        let view = surface.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );
        return TextureContainer {
            width: size.0,
            height: size.1,
            view,
            bind_groups: Vec::with_capacity(0)
        };
    }
}

impl TextureContainer {
    pub fn size(&self) -> (u32,u32) {
        (self.width,self.height)
    }

    pub fn get_bind_group(&self,filter_mode: FilterMode,address_mode: AddressMode) -> Option<&BindGroup> {
        self.bind_groups.get(get_bind_group_index(filter_mode,address_mode))
    }
}
