use std::collections::VecDeque;
use std::default;
use std::marker::PhantomData;
use std::fmt::{self,Write};

use crate::{UWimpyPoint, WimpyColor, WimpyNamedColor, WimpyOpacity, WimpyRect, WimpyRectQuadrant, WimpyVec, WimpyVecAxis};
use crate::app::graphics::{DrawData2D, LinePoint, fonts::*};
use crate::app::graphics::{MutableFrame,RenderPassBuilder,TextDirection,TextRenderConfig};
use crate::collections::StringPool;

const TEXT_MARGIN: f32 = 5.0;

const LOG_LINE_SIZE: usize = 64;
const LOG_LINE_COUNT: usize = 8;
const GRAPH_BUFFER_SIZE: usize = 1024;

const LOG_CHANNEL_COUNT: usize = 5;
const GRAPH_CHANNEL_COUNT: usize = 6;
const LABEL_CHANNEL_COUNT: usize = 8;

const RECIP_255: f32 = 1.0 / 255.0;

type LogStringPool = StringPool<LOG_LINE_SIZE>;

#[derive(Default)]
pub struct DebugShell {
    string_pool: LogStringPool,
    log_buffers: LogBufferSet,
    graph_buffers: GraphBufferSet,
    render_config: DebugRenderConfig,
    labels: LabelSet,
    log_display: LogDisplay,
    buffer: DebugRenderBuffer
}

pub struct LogBuffer {
    value: VecDeque<String>
}

pub struct GraphBuffer {
    value: VecDeque<i8>
}

#[derive(Default)]
struct BufferSet<const SIZE: usize,T,TChannel>
where
    [T;SIZE]: Default,
{
    buffers: [T;SIZE],
    _phantom: PhantomData<TChannel>
}

impl<const SIZE: usize,T,TChannel> BufferSet<SIZE,T,TChannel>
where
    [T;SIZE]: Default,
    TChannel: Into<usize>
{
    fn get(&self,channel: TChannel) -> &T {
        return &self.buffers[channel.into()];
    }
    fn get_mut(&mut self,channel: TChannel) -> &mut T {
        return &mut self.buffers[channel.into()];
    }
}

type GraphBufferSet = BufferSet<GRAPH_CHANNEL_COUNT,GraphBuffer,GraphChannelID>;
#[derive(Default,Copy,Clone)]
pub enum GraphChannelID {
    #[default]
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5
}
impl From<GraphChannelID> for usize {
    fn from(value: GraphChannelID) -> Self {
        value as usize
    }
}

type LabelSet = BufferSet<LABEL_CHANNEL_COUNT,String,LabelChannelID>;
#[derive(Default,Copy,Clone)]
pub enum LabelChannelID {
    #[default]
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5,
    Seven = 6,
    Eight = 7
}
impl From<LabelChannelID> for usize {
    fn from(value: LabelChannelID) -> Self {
        value as usize
    }
}


type LogBufferSet = BufferSet<LOG_CHANNEL_COUNT,LogBuffer,LogChannelID>;
#[derive(Default,Copy,Clone)]
pub enum LogChannelID {
    Trace = 0,
    Debug = 1,
    #[default]
    Info = 2,
    Warn = 3,
    Error = 4
}
impl From<LogChannelID> for usize {
    fn from(value: LogChannelID) -> Self {
        value as usize
    }
}

impl Default for GraphBuffer {
    fn default() -> Self {
        Self {
            value: VecDeque::with_capacity(GRAPH_BUFFER_SIZE)
        }
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        return Self {
            value: VecDeque::with_capacity(LOG_LINE_COUNT)
        }
    }
}

impl GraphBuffer {
    pub fn push(&mut self,value: i8) {
        if self.value.len() == self.value.capacity() {
            self.value.pop_front();
        }
        self.value.push_front(value);
    }
}

impl LogBuffer {
    pub fn push(&mut self,string_pool: &mut LogStringPool,text: &str) {
        if self.value.len() == self.value.capacity() {
            if let Some(stale_entry) = self.value.pop_front() {
                string_pool.return_item(stale_entry);
            }
            let mut new_entry = string_pool.take_item();
            new_entry.push_str(text);
            self.value.push_back(new_entry);
        }
    }
}

#[derive(Default,Copy,Clone)]
pub enum GraphWidth {
    #[default]
    Full,
    Half,
    Quarter,
    Eighth
}

impl Into<usize> for GraphWidth {
    fn into(self) -> usize {
        use GraphWidth::*;
        match self {
            Full => GRAPH_BUFFER_SIZE,
            Half => GRAPH_BUFFER_SIZE / 2,
            Quarter => GRAPH_BUFFER_SIZE / 4,
            Eighth => GRAPH_BUFFER_SIZE / 8,
        }
    }
}

#[derive(Default,Copy,Clone)]
pub struct GraphLayer {
    pub color: WimpyNamedColor,
    pub id: GraphChannelID,
}

#[derive(Copy,Clone)]
pub enum GraphLayers {
    SingleLayer {
        layers: [GraphLayer;1]
    },
    DualLayer {
        layers: [GraphLayer;2]
    },
    TriLayer {
        layers: [GraphLayer;3]
    },
    QuadLayer {
        layers: [GraphLayer;4]
    },
}

impl GraphLayers {
    pub fn channels(&self) -> &[GraphLayer] {
        use GraphLayers::*;
        match self {
            SingleLayer { layers } => layers,
            DualLayer { layers } => layers,
            TriLayer { layers } => layers,
            QuadLayer { layers } => layers,
        }
    }
}

impl Default for GraphLayers {
    fn default() -> Self {
        Self::SingleLayer {
            layers: Default::default()
        }
    }
}

#[derive(Default,Copy,Clone)]
pub enum PaneItem {
    #[default]
    None,
    Graph {
        background_color: WimpyNamedColor,
        background_opacity: WimpyOpacity,
        width: GraphWidth,
        layers: GraphLayers
    },
    Label {
        background_color: WimpyNamedColor,
        background_opacity: WimpyOpacity,
        text_color: WimpyNamedColor,
        channel: LabelChannelID
    }
}

#[derive(Default,Copy,Clone)]
pub enum PaneLayout {
    #[default]
    None,
    One {
        items: [PaneItem;1],
    },
    Two {
        items: [PaneItem;2],
        axis: WimpyVecAxis,
    },
    Three {
        items: [PaneItem;3],
        axis: WimpyVecAxis,
    },
    Four {
        items: [PaneItem;4],
        axis: WimpyVecAxis,
    },
    FourQuadrant {
        items: [PaneItem;4],
    },
}

impl PaneLayout {
    pub fn items(&self) -> &[PaneItem] {
        use PaneLayout::*;
        match self {
            None => &[],
            One { items } => items,
            Two { items, ..} => items,
            Three { items, .. } => items,
            Four { items, .. } => items,
            FourQuadrant { items } => items,
        }
    }
}

#[derive(Default)]
pub struct PaneConfig {
    pub layout: PaneLayout,
    pub background_color: WimpyColor,
    pub size: WimpyVec
}

#[derive(Default)]
pub struct DebugRenderConfig {
    pub top_left: PaneConfig,
    pub top_right: PaneConfig,
    pub bottom_left: PaneConfig,
    pub bottom_right: PaneConfig,
}

#[derive(Default)]
pub enum LogDisplay {
    #[default]
    None,
    Some {
        trace: bool,
        debug: bool,
        info: bool,
        warn: bool,
        error: bool,
    }
}

impl DebugShell {
    pub fn set_log_display(&mut self,display: LogDisplay) {
        self.log_display = display;
    }

    pub fn set_render_config(&mut self,config: DebugRenderConfig) {
        self.render_config = config;
    }

    pub fn set_render_config_top_left(&mut self,config: PaneConfig) {
        self.render_config.top_left = config;
    }

    pub fn set_render_config_top_right(&mut self,config: PaneConfig) {
        self.render_config.top_right = config;
    }

    pub fn set_render_config_bottom_left(&mut self,config: PaneConfig) {
        self.render_config.bottom_left = config;
    }

    pub fn set_render_config_bottom_right(&mut self,config: PaneConfig) {
        self.render_config.bottom_right = config;
    }

    pub fn clear_log_display(&mut self) {
        self.log_display = Default::default();
    }

    pub fn clear_render_config(&mut self) {
        self.render_config = Default::default();
    }

    pub fn log(&mut self,channel: LogChannelID,text: &str) {
        let log_buffer = self.log_buffers.get_mut(channel);
        log_buffer.push(&mut self.string_pool,text);
    }

    pub fn set_graph_value(&mut self,channel: GraphChannelID,value: i8) {
        let graph_buffer = self.graph_buffers.get_mut(channel);
        graph_buffer.push(value);
    }

    pub fn set_label_text(&mut self,channel: LabelChannelID,value: &str) {
        let label = self.labels.get_mut(channel);
        label.clear();
        label.push_str(value);
    }

    pub fn set_label_text_fmt<'a>(&'a mut self,channel: LabelChannelID,args: fmt::Arguments){
        let label = self.labels.get_mut(channel);
        label.clear();
        let _ = label.write_fmt(args);
    }

    pub fn clear_label(&mut self,channel: LabelChannelID) {
        let label = self.labels.get_mut(channel);
        label.clear();
    }

    fn draw_graphs<'a,'rp,TFrame>(&'a mut self,render_pass: &'a mut RenderPassBuilder<'rp,TFrame>)
    where
        TFrame: MutableFrame
    {
        let mut background_fill_pass = render_pass.set_pipeline_2d();
        background_fill_pass.draw_untextured(self.buffer.graph_commands.iter().map(|graph|DrawData2D{
            destination: graph.area,
            source: WimpyRect::ONE,
            color: WimpyColor::BLACK,
            rotation: 0.0,
        }));
        let mut line_pass = render_pass.set_pipeline_lines();
        for graph in &self.buffer.graph_commands {
            for channel_config in graph.layers.channels() {
                let buffer = &self.graph_buffers.get(channel_config.id).value;

                let sample_count: usize = graph.width.into();

                let x_step = graph.area.width() / sample_count as f32;
                let y_step = RECIP_255 * graph.area.height();

                let color = channel_config.color.into();
                line_pass.draw(buffer.iter().take(sample_count).enumerate().map(|(index,value)|{
                    LinePoint {
                        point: WimpyVec {
                            x: (index as f32).mul_add(-x_step,graph.area.x()),
                            y: (*value as f32 + 128.0).mul_add(y_step,graph.area.y())
                        },
                        color,
                    }
                }));
            }
        }
    }

    fn draw_labels<'a,'rp,TFrame>(&'a mut self,render_pass: &'a mut RenderPassBuilder<'rp,TFrame>)
    where
        TFrame: MutableFrame
    {
        todo!();
    }

    pub fn render<'a,'rp,TFrame>(&'a mut self,render_pass: &'a mut RenderPassBuilder<'rp,TFrame>)
    where
        TFrame: MutableFrame
    {
        let frame_size = WimpyVec::from(render_pass.frame().size());
        {
            let tl_size = self.render_config.top_left.size;
            self.buffer.queue_pane(self.render_config.top_left.layout,TextDirection::LeftToRight,WimpyRect {
                position: WimpyVec::ZERO,
                size: tl_size,
            });
        }
        {
            let tr_size = self.render_config.top_right.size;
            self.buffer.queue_pane(self.render_config.top_right.layout,TextDirection::RightToLeft,WimpyRect {
                position: WimpyVec::from([frame_size.x - tr_size.x,0.0]),
                size: tr_size,
            });
        }
        {
            let bl_size = self.render_config.bottom_left.size;
            self.buffer.queue_pane(self.render_config.bottom_left.layout,TextDirection::LeftToRight,WimpyRect {
                position: WimpyVec::from([0.0,frame_size.y - bl_size.y]),
                size: bl_size,
            });
        }
        {
            let br_size = self.render_config.bottom_right.size;
            self.buffer.queue_pane(self.render_config.bottom_right.layout,TextDirection::RightToLeft,WimpyRect {
                position: WimpyVec::from([frame_size.x - br_size.x,frame_size.y - br_size.y]),
                size: br_size,
            });
        }
        self.draw_graphs(render_pass);
        self.draw_labels(render_pass);
        self.buffer.reset();
    }
}

struct DrawBackgroundCommand {
    background_color: WimpyNamedColor,
    background_opacity: WimpyOpacity,
    area: WimpyRect
}

struct DebugRenderBuffer {
    draw_background_command: Vec<DrawBackgroundCommand>,
    graph_commands: Vec<DrawGraphCommand>,
    label_commands: Vec<DrawLabelCommand>
}

impl Default for DebugRenderBuffer {
    fn default() -> Self {
        Self {
            draw_background_command: Vec::with_capacity(Self::COMMAND_CAPACITY),
            graph_commands: Vec::with_capacity(Self::COMMAND_CAPACITY),
            label_commands: Vec::with_capacity(Self::COMMAND_CAPACITY)
        }
    }
}

impl DebugRenderBuffer {
    const NUM_OF_PANES: usize = 4;
    const MAX_PANE_SECTORS: usize = 4;
    const COMMAND_CAPACITY: usize = Self::NUM_OF_PANES * Self::MAX_PANE_SECTORS;
}

struct DrawGraphCommand {
    layers: GraphLayers,
    width: GraphWidth,
    area: WimpyRect,
}

struct DrawLabelCommand {
    channel: LabelChannelID,
    text_color: WimpyNamedColor,
    position: WimpyVec,
    direction: TextDirection,
}

const QUADRANT_ORDER: [WimpyRectQuadrant;4] = [
    WimpyRectQuadrant::TopLeft,
    WimpyRectQuadrant::TopRight,
    WimpyRectQuadrant::BottomLeft,
    WimpyRectQuadrant::BottomRight
];

impl DebugRenderBuffer {
    fn reset(&mut self) {
        self.graph_commands.clear();
        self.label_commands.clear();
    }

    fn queue_graph(
        &mut self,
        background_color: WimpyNamedColor,
        background_opacity: WimpyOpacity,
        width: GraphWidth,
        layers: GraphLayers,
        area: WimpyRect
    ) {
        self.graph_commands.push(DrawGraphCommand {
            layers,
            width,
            area
        });
    }

    fn queue_label(&mut self,channel: LabelChannelID,color: WimpyNamedColor,direction: TextDirection,position: WimpyVec) {
        self.label_commands.push(DrawLabelCommand {
            channel,
            text_color: color,
            position,
            direction
        });
    }

    fn queue_pane(&mut self,pane: PaneLayout,text_direction: TextDirection,area: WimpyRect) {
        let items = pane.items();
        match pane {
            PaneLayout::None => {},
            PaneLayout::One { items } => match items[0] {
                PaneItem::None => {},
                PaneItem::Graph {
                    layers,
                    width
                } => self.queue_graph(layers,background_color,width,area),
                PaneItem::Label {
                    channel,
                    text_color: color
                } => self.queue_label(channel,color,text_direction,area.position + TEXT_MARGIN),
            },
            PaneLayout::Two { axis, ..} | PaneLayout::Three { axis, ..} | PaneLayout::Four { axis, .. } => {
                let (size,stride) = {
                    let length = area.size[axis] * (1.0 / items.len() as f32);
                    let stride = WimpyVec::from_axis(axis,length);
                    let mut size = area.size;
                    size[axis] *= length;
                    (size,stride)
                };
                for (i,item) in items.iter().copied().enumerate() {
                    let position = area.position + stride * i as f32;
                    match item {
                        PaneItem::None => {},
                        PaneItem::Graph {
                            layers,
                            width
                        } => self.queue_graph(layers,width,WimpyRect {position,size}),
                        PaneItem::Label {
                            channel,
                            text_color: color
                        } => self.queue_label(channel,color,text_direction,position + TEXT_MARGIN),
                    }
                }
            },
            PaneLayout::FourQuadrant { items } => {
                for (i,item) in items.iter().copied().enumerate() {
                    let quadrant = area.quadrant(QUADRANT_ORDER[i]);
                    match item {
                        PaneItem::None => {},
                        PaneItem::Graph {
                            layers,
                            width
                        } => self.queue_graph(layers,width,quadrant),
                        PaneItem::Label {
                            channel,
                            text_color: color
                        } => self.queue_label(channel,color,text_direction,quadrant.position + TEXT_MARGIN),
                    }
                }
            },
        }
    }
}
