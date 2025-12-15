use image::{
    DynamicImage,
    EncodableLayout,
    GenericImageView
};

use wgpu::{
    BindGroup, BindGroupLayout, Device, SurfaceTexture, TextureUsages, TextureView
};

use crate::{
    frame::{
        FilterMode,
        WrapMode,
    },
    wgpu_interface::WGPUInterface
};

pub struct TextureContainer {
    width: u32,
    height: u32,
    view: TextureView,
    bind_groups: Vec<BindGroup>
}

pub enum SamplerMode {
    NearestClamp = 0,
    NearestRepeat = 1,
    NearestMirrorRepeat = 2,
    LinearClamp = 3,
    LinearRepeat = 4,
    LinearMirrorRepeat = 5
}

impl SamplerMode {
    pub fn get_mode(filter_mode: FilterMode,wrap_mode: WrapMode) -> SamplerMode {
        return match (filter_mode,wrap_mode) {
            (FilterMode::Nearest,WrapMode::Clamp) => SamplerMode::NearestClamp,
            (FilterMode::Nearest,WrapMode::Repeat) => SamplerMode::NearestRepeat,
            (FilterMode::Nearest,WrapMode::MirrorRepeat) => SamplerMode::NearestMirrorRepeat,
            (FilterMode::Linear,WrapMode::Clamp) => SamplerMode::LinearClamp,
            (FilterMode::Linear,WrapMode::Repeat) => SamplerMode::LinearRepeat,
            (FilterMode::Linear,WrapMode::MirrorRepeat) => SamplerMode::LinearMirrorRepeat,
        };
    }
}

fn get_bind_groups(texture_view: &TextureView,device: &Device,bind_group_layout: &BindGroupLayout) -> Vec<BindGroup> {

    use wgpu::{FilterMode,AddressMode};
    let bind_groups: Vec<BindGroup> = vec![
        /* Must match index order of SamplerMode */
        (FilterMode::Nearest,AddressMode::ClampToEdge),
        (FilterMode::Nearest,AddressMode::Repeat),
        (FilterMode::Nearest,AddressMode::MirrorRepeat),
        (FilterMode::Linear,AddressMode::ClampToEdge),
        (FilterMode::Linear,AddressMode::Repeat),
        (FilterMode::Linear,AddressMode::MirrorRepeat),
    ].iter().map(|(filter,address)|{
        let (a,f) = (*address,*filter);
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: a,
            address_mode_v: a,
            address_mode_w: a,
            mag_filter: f,
            min_filter: f,
            mipmap_filter: f,
            ..Default::default()
        })
    }).map(|sampler|device.create_bind_group(&wgpu::BindGroupDescriptor {
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
    })).collect();
    return bind_groups;
}

fn validate_dimensions(dimensions: (u32,u32)) {
    if dimensions.0 > 0 && dimensions.1 > 0 {
        return;
    }

    //TODO: Validate max dimension size with WGPU capabilities

    panic!("Invalid texture container dimensions. Dimensions must be greater than 0.");
}

struct TextureCreationParameters {
    dimensions: (u32,u32),
    mutable: bool
}

fn create_texture(
    wgpu_interface: &impl WGPUInterface,
    bind_group_layout: &BindGroupLayout,
    image_data: Option<&[u8]>,
    parameters: TextureCreationParameters
) -> TextureContainer {

    let dimensions = parameters.dimensions;

    validate_dimensions(dimensions);

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let mut usage_flags = TextureUsages::TEXTURE_BINDING;

    if image_data.is_some() {
        usage_flags |= TextureUsages::COPY_DST;
    }

    if parameters.mutable {
        usage_flags |= TextureUsages::RENDER_ATTACHMENT;
    }

    let device = wgpu_interface.get_device();
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
        wgpu_interface.get_queue().write_texture(
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
                bytes_per_row: Some(4*dimensions.0), 
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );
    }

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_groups = get_bind_groups(&view,&device,bind_group_layout);

    return TextureContainer {
        width: dimensions.0,
        height: dimensions.1,
        view,
        bind_groups
    };
}

impl TextureContainer {

    pub fn get_view(&self) -> &TextureView {
        return &self.view;
    }

    pub fn create_mutable(wgpu_interface: &impl WGPUInterface,bind_group_layout: &BindGroupLayout,dimensions: (u32,u32)) -> TextureContainer {
        return create_texture(wgpu_interface,bind_group_layout,None,TextureCreationParameters {
            dimensions,
            mutable: true
        });
    }

    pub fn from_image(wgpu_interface: &impl WGPUInterface,bind_group_layout: &BindGroupLayout,image: &DynamicImage) -> TextureContainer {
        let dimensions = image.dimensions();

        //TODO: Make sure alpha channel is premultiplied ... Somehow.
        let image_data = image.to_rgba8();

        return create_texture(wgpu_interface,bind_group_layout,Some(image_data.as_bytes()),TextureCreationParameters {
            dimensions,
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
        return (self.width,self.height);
    }

    pub fn get_bind_group(&self,sampler_mode: SamplerMode) -> &BindGroup {
        if let Some(bind_group) = self.bind_groups.get(sampler_mode as usize) {
            return bind_group;
        } else {
            panic!("Bind group not found for this sampler mode.");
        }
    }
}
