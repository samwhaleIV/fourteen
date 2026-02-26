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

type LogStringPool = StringPool<LOG_LINE_SIZE>;

#[derive(Default)]
pub struct DebugShell {
    string_pool: LogStringPool,
    log_buffers: LogBufferSet,
    graph_buffers: GraphBufferSet,
    render_config: DebugRenderConfig,
    labels: LabelSet,
    log_display: LogDisplay,
    buffers: DebugRenderBuffers
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
        &self.buffers[channel.into()]
    }
    fn get_mut(&mut self,channel: TChannel) -> &mut T {
        &mut self.buffers[channel.into()]
    }
}

type GraphBufferSet = BufferSet<GRAPH_CHANNEL_COUNT,GraphBuffer,GraphID>;
#[derive(Default,Copy,Clone)]
pub enum GraphID {
    #[default]
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5
}
impl From<GraphID> for usize {
    fn from(value: GraphID) -> Self {
        value as usize
    }
}

type LabelSet = BufferSet<LABEL_CHANNEL_COUNT,String,LabelID>;
#[derive(Default,Copy,Clone)]
pub enum LabelID {
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
impl From<LabelID> for usize {
    fn from(value: LabelID) -> Self {
        value as usize
    }
}


type LogBufferSet = BufferSet<LOG_CHANNEL_COUNT,LogBuffer,LogID>;
#[derive(Default,Copy,Clone)]
pub enum LogID {
    Trace = 0,
    Debug = 1,
    #[default]
    Info = 2,
    Warn = 3,
    Error = 4
}
impl From<LogID> for usize {
    fn from(value: LogID) -> Self {
        value as usize
    }
}

impl Default for GraphBuffer {
    fn default() -> Self {
        Self {
            value: VecDeque::from(vec![0;GRAPH_BUFFER_SIZE])
        }
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self {
            value: VecDeque::with_capacity(LOG_LINE_COUNT)
        }
    }
}

impl GraphBuffer {
    pub fn push(&mut self,value: i8) {
        if self.value.len() == self.value.capacity() {
            self.value.pop_back();
        }
        self.value.push_front(value);
    }
}

impl LogBuffer {
    pub fn push(&mut self,string_pool: &mut LogStringPool,text: &str) {
        if self.value.len() == self.value.capacity() {
            if let Some(stale_entry) = self.value.pop_back() {
                string_pool.return_item(stale_entry);
            }
            let mut new_entry = string_pool.take_item();
            new_entry.push_str(text);
            self.value.push_front(new_entry);
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
    pub id: GraphID,
    pub color: WimpyNamedColor,
}

#[derive(Copy,Clone)]
pub enum GraphLayers {
    Single {
        layers: [GraphLayer;1]
    },
    Dual {
        layers: [GraphLayer;2]
    },
    Tri {
        layers: [GraphLayer;3]
    },
    Quad {
        layers: [GraphLayer;4]
    },
}

impl GraphLayers {
    pub fn single(graph_layer: GraphLayer) -> Self {
        Self::Single {
            layers: [graph_layer]
        }
    }
}

impl GraphLayers {
    pub fn channels(&self) -> &[GraphLayer] {
        use GraphLayers::*;
        match self {
            Single { layers } => layers,
            Dual { layers } => layers,
            Tri { layers } => layers,
            Quad { layers } => layers,
        }
    }
}

impl Default for GraphLayers {
    fn default() -> Self {
        Self::Single {
            layers: Default::default()
        }
    }
}

#[derive(Default,Copy,Clone)]
pub enum PaneItem {
    #[default]
    None,
    Graph {
        width: GraphWidth,
        layers: GraphLayers
    },
    Label {
        color: WimpyNamedColor,
        channel: LabelID
    }
}

#[derive(Default,Copy,Clone)]
pub struct SubPane {
    pub item: PaneItem,
    pub background_color: WimpyNamedColor,
    pub background_opacity: WimpyOpacity,
}

#[derive(Default,Copy,Clone)]
pub enum PaneLayout {
    #[default]
    None,
    Single {
        panes: [SubPane;1],
    },
    DivTwo {
        panes: [SubPane;2],
        axis: WimpyVecAxis,
    },
    DivThree {
        panes: [SubPane;3],
        axis: WimpyVecAxis,
    },
    DivFour {
        panes: [SubPane;4],
        axis: WimpyVecAxis,
    },
    Quadrants {
        panes: [SubPane;4],
    },
}

#[derive(Default,Copy,Clone)]
pub struct Pane {
    pub size: WimpyVec,
    pub layout: PaneLayout
}

impl PaneLayout {
    pub fn single(sub_pane: SubPane) -> Self {
        Self::Single {
            panes: [sub_pane]
        }
    }
    pub fn panes(&self) -> &[SubPane] {
        use PaneLayout::*;
        match self {
            None => &[],
            Single { panes } => panes,
            DivTwo { panes, ..} => panes,
            DivThree { panes, .. } => panes,
            DivFour { panes, .. } => panes,
            Quadrants { panes } => panes,
        }
    }
}

#[derive(Default)]
pub struct DebugRenderConfig {
    pub top_left: Pane,
    pub top_right: Pane,
    pub bottom_left: Pane,
    pub bottom_right: Pane,
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

    pub fn set_render_config(&mut self,config: DebugRenderConfig) {
        self.render_config = config;
    }

    pub fn get_render_config(&mut self) -> &mut DebugRenderConfig {
        &mut self.render_config
    }

    pub fn clear_render_config(&mut self) {
        self.render_config = Default::default();
    }

    pub fn set_log_display(&mut self,display: LogDisplay) {
        self.log_display = display;
    }

    pub fn clear_log_display(&mut self) {
        self.log_display = Default::default();
    }

    pub fn log(&mut self,channel: LogID,text: &str) {
        let log_buffer = self.log_buffers.get_mut(channel);
        log_buffer.push(&mut self.string_pool,text);
    }

    pub fn set_graph_value(&mut self,channel: GraphID,value: i8) {
        let graph_buffer = self.graph_buffers.get_mut(channel);
        graph_buffer.push(value);
    }

    pub fn set_label_text(&mut self,channel: LabelID,value: &str) {
        let label = self.labels.get_mut(channel);
        label.clear();
        label.push_str(value);
    }

    pub fn set_label_text_fmt<'a>(&'a mut self,channel: LabelID,args: fmt::Arguments){
        let label = self.labels.get_mut(channel);
        label.clear();
        let _ = label.write_fmt(args);
    }

    pub fn clear_label(&mut self,channel: LabelID) {
        let label = self.labels.get_mut(channel);
        label.clear();
    }

    fn draw_graphs<'a,'rp,TFrame>(&'a mut self,render_pass: &'a mut RenderPassBuilder<'rp,TFrame>)
    where
        TFrame: MutableFrame
    {
        let mut line_pass = render_pass.set_pipeline_lines();
        for graph in &self.buffers.graph_commands {
            for channel_config in graph.layers.channels() {
                let buffer = &self.graph_buffers.get(channel_config.id).value;

                let sample_count: usize = graph.width.into();

                let x_step = graph.area.width() / (sample_count - 1) as f32;
                let y_step =  graph.area.height() / 255.0;

                let color = channel_config.color.into();
                let right_edge = graph.area.right();

                line_pass.draw(buffer.iter().take(sample_count).enumerate().map(|(index,value)|{
                    LinePoint {
                        point: WimpyVec {
                            x: (index as f32).mul_add(-x_step,right_edge),
                            y: (*value as f32 + 127.5).mul_add(y_step,graph.area.y())
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
        //todo!();
    }

    fn draw_backgrounds<'a,'rp,TFrame>(&'a mut self,render_pass: &'a mut RenderPassBuilder<'rp,TFrame>)
    where
        TFrame: MutableFrame
    {
        if self.buffers.background_commands.len() <= 0 {
            return;
        }
        let mut pipeline = render_pass.set_pipeline_2d();
        pipeline.draw_untextured(self.buffers.background_commands.iter().map(|command|{
            DrawData2D {
                destination: command.area,
                source: WimpyRect::ONE,
                color: (command.background_color,command.background_opacity).into_linear(),
                rotation: 0.0,
            }
        }));
    }

    pub fn render<'a,'rp,TFrame>(&'a mut self,render_pass: &'a mut RenderPassBuilder<'rp,TFrame>)
    where
        TFrame: MutableFrame
    {
        let frame_size = WimpyVec::from(render_pass.frame().size());
        {
            let tl_size = self.render_config.top_left.size;
            self.buffers.queue_pane(self.render_config.top_left.layout,TextDirection::LeftToRight,WimpyRect {
                position: WimpyVec::ZERO,
                size: tl_size,
            });
        }
        {
            let tr_size = self.render_config.top_right.size;
            self.buffers.queue_pane(self.render_config.top_right.layout,TextDirection::RightToLeft,WimpyRect {
                position: WimpyVec::from([frame_size.x - tr_size.x,0.0]),
                size: tr_size,
            });
        }
        {
            let bl_size = self.render_config.bottom_left.size;
            self.buffers.queue_pane(self.render_config.bottom_left.layout,TextDirection::LeftToRight,WimpyRect {
                position: WimpyVec::from([0.0,frame_size.y - bl_size.y]),
                size: bl_size,
            });
        }
        {
            let br_size = self.render_config.bottom_right.size;
            self.buffers.queue_pane(self.render_config.bottom_right.layout,TextDirection::RightToLeft,WimpyRect {
                position: WimpyVec::from([frame_size.x - br_size.x,frame_size.y - br_size.y]),
                size: br_size,
            });
        }
        self.draw_backgrounds(render_pass);
        self.draw_graphs(render_pass);
        self.draw_labels(render_pass);

        self.buffers.reset();
    }
}

struct DrawBackgroundCommand {
    background_color: WimpyNamedColor,
    background_opacity: WimpyOpacity,
    area: WimpyRect
}

struct DebugRenderBuffers {
    background_commands: Vec<DrawBackgroundCommand>,
    graph_commands: Vec<DrawGraphCommand>,
    label_commands: Vec<DrawLabelCommand>
}

impl Default for DebugRenderBuffers {
    fn default() -> Self {
        Self {
            background_commands: Vec::with_capacity(Self::COMMAND_CAPACITY),
            graph_commands: Vec::with_capacity(Self::COMMAND_CAPACITY),
            label_commands: Vec::with_capacity(Self::COMMAND_CAPACITY)
        }
    }
}

impl DebugRenderBuffers {
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
    channel: LabelID,
    color: WimpyNamedColor,
    area: WimpyRect,
    direction: TextDirection,
}

const QUADRANT_ORDER: [WimpyRectQuadrant;4] = [
    WimpyRectQuadrant::TopLeft,
    WimpyRectQuadrant::TopRight,
    WimpyRectQuadrant::BottomLeft,
    WimpyRectQuadrant::BottomRight
];

impl DebugRenderBuffers {
    fn reset(&mut self) {
        self.background_commands.clear();
        self.graph_commands.clear();
        self.label_commands.clear();
    }

    fn queue_background_color(&mut self,pane: &SubPane,command: DrawBackgroundCommand) {
        if let WimpyOpacity::Transparent = pane.background_opacity {
            return;
        }
        self.background_commands.push(command);
    }

    fn queue_pane(&mut self,pane: PaneLayout,direction: TextDirection,area: WimpyRect) {
        let items = pane.panes();
        match pane {
            PaneLayout::None => {},
            PaneLayout::Single { panes } => {
                let pane = panes[0];
                self.queue_background_color(&pane,DrawBackgroundCommand {
                    background_color: pane.background_color,
                    background_opacity: pane.background_opacity,
                    area
                });
                match pane.item {
                    PaneItem::None => {},
                    PaneItem::Graph { width, layers } => {
                        self.graph_commands.push(DrawGraphCommand {
                            layers,
                            width,
                            area
                        });
                    },
                    PaneItem::Label { color, channel } => {
                        self.label_commands.push(DrawLabelCommand {
                            channel,
                            color,
                            direction,
                            area
                        });
                    },
                }
            },
            PaneLayout::DivTwo { axis, ..} | PaneLayout::DivThree { axis, ..} | PaneLayout::DivFour { axis, .. } => {
                let (size,stride) = {
                    let length = area.size[axis] * (1.0 / items.len() as f32);
                    let stride = WimpyVec::from_axis(axis,length);
                    let mut size = area.size;
                    size[axis] *= length;
                    (size,stride)
                };
                for (i,pane) in items.iter().copied().enumerate() {
                    self.queue_background_color(&pane,DrawBackgroundCommand {
                        background_color: pane.background_color,
                        background_opacity: pane.background_opacity,
                        area
                    });
                    let sub_area = WimpyRect {
                        position: area.position + stride * i as f32,
                        size
                    };
                    match pane.item {
                        PaneItem::None => {},
                        PaneItem::Graph { width, layers } => {
                            self.graph_commands.push(DrawGraphCommand {
                                layers,
                                width,
                                area: sub_area
                            });
                        },
                        PaneItem::Label { color: text_color, channel } => {
                            self.label_commands.push(DrawLabelCommand {
                                channel,
                                color: text_color,
                                area: sub_area,
                                direction
                            })
                        },
                    }
                }
            },
            PaneLayout::Quadrants { panes: items } => {
                for (i,pane) in items.iter().copied().enumerate() {
                    self.queue_background_color(&pane,DrawBackgroundCommand {
                        background_color: pane.background_color,
                        background_opacity: pane.background_opacity,
                        area
                    });
                    let sub_area = area.quadrant(QUADRANT_ORDER[i]);
                    match pane.item {
                        PaneItem::None => {},
                        PaneItem::Graph { layers, width } => {
                            self.graph_commands.push(DrawGraphCommand {
                                layers,
                                width,
                                area: sub_area
                            });
                        },
                        PaneItem::Label { channel, color } => {
                            self.label_commands.push(DrawLabelCommand {
                                channel,
                                color,
                                area: sub_area,
                                direction
                            })
                        },
                    }
                }
            },
        }
    }
}
