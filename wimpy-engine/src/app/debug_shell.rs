use std::collections::VecDeque;
use std::marker::PhantomData;
use std::fmt::{self,Write};

use crate::WimpyColor;
use crate::app::graphics::fonts::*;
use crate::app::graphics::{MutableFrame, RenderPassBuilder, TextRenderBehavior, TextRenderConfig};
use crate::collections::StringPool;

const LOG_LINE_SIZE: usize = 64;
const LOG_LINE_COUNT: usize = 8;
const GRAPH_BUFFER_SIZE: usize = 256;

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
    labels: LabelSet
}

pub struct LogBuffer {
    buffer: VecDeque<String>
}

pub struct GraphBuffer {
    buffer: VecDeque<u8>
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

type GraphBufferSet = BufferSet<GRAPH_CHANNEL_COUNT,GraphBuffer,GraphChannel>;
#[derive(Default,Copy,Clone)]
pub enum GraphChannel {
    #[default]
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5
}
impl From<GraphChannel> for usize {
    fn from(value: GraphChannel) -> Self {
        value as usize
    }
}

type LabelSet = BufferSet<LABEL_CHANNEL_COUNT,String,LabelChannel>;
#[derive(Default,Copy,Clone)]
pub enum LabelChannel {
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
impl From<LabelChannel> for usize {
    fn from(value: LabelChannel) -> Self {
        value as usize
    }
}


type LogBufferSet = BufferSet<LOG_CHANNEL_COUNT,LogBuffer,LogChannel>;
#[derive(Default,Copy,Clone)]
pub enum LogChannel {
    Trace = 0,
    Debug = 1,
    #[default]
    Info = 2,
    Warn = 3,
    Error = 4
}
impl From<LogChannel> for usize {
    fn from(value: LogChannel) -> Self {
        value as usize
    }
}

impl Default for GraphBuffer {
    fn default() -> Self {
        Self {
            buffer: VecDeque::with_capacity(GRAPH_BUFFER_SIZE)
        }
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        return Self {
            buffer: VecDeque::with_capacity(LOG_LINE_COUNT)
        }
    }
}

impl GraphBuffer {
    pub fn push(&mut self,value: u8) {
        if self.buffer.len() == self.buffer.capacity() {
            self.buffer.pop_front();
        }
        self.buffer.push_front(value);
    }
}

impl LogBuffer {
    pub fn push(&mut self,string_pool: &mut LogStringPool,text: &str) {
        if self.buffer.len() == self.buffer.capacity() {
            if let Some(stale_entry) = self.buffer.pop_front() {
                string_pool.return_item(stale_entry);
            }
            let mut new_entry = string_pool.take_item();
            new_entry.push_str(text);
            self.buffer.push_back(new_entry);
        }
    }
}

#[derive(Default)]
pub enum GraphWidth {
    #[default]
    Full,
    Half,
    Quarter,
    Eighth
}

#[derive(Default)]
pub enum PaneItem {
    #[default]
    None,
    Graph {
        channel: GraphChannel,
        width: GraphWidth,
    },
    Log {
        channel: LogChannel,
    },
    Label {
        channel: LabelChannel,
    }
}

#[derive(Default)]
pub enum PaneLayout {
    #[default]
    None,
    One {
        items: [PaneItem;1],
    },
    Two {
        items: [PaneItem;2],
    },
    Three {
        items: [PaneItem;3],
    },
    Four {
        items: [PaneItem;4],
    },
}

#[derive(Default)]
pub struct DebugRenderConfig {
    pub top_left: PaneLayout,
    pub top_right: PaneLayout,
    pub bottom_left: PaneLayout,
    pub bottom_right: PaneLayout,
}

impl DebugShell {
    pub fn set_render_config(&mut self,config: DebugRenderConfig) {
        self.render_config = config;
    }

    pub fn set_render_config_top_left(&mut self,layout: PaneLayout) {
        self.render_config.top_left = layout;
    }

    pub fn set_render_config_top_right(&mut self,layout: PaneLayout) {
        self.render_config.top_right = layout;
    }

    pub fn set_render_config_bottom_left(&mut self,layout: PaneLayout) {
        self.render_config.bottom_left = layout;
    }

    pub fn set_render_config_bottom_right(&mut self,layout: PaneLayout) {
        self.render_config.bottom_right = layout;
    }

    pub fn log(&mut self,channel: LogChannel,text: &str) {
        let log_buffer = self.log_buffers.get_mut(channel);
        log_buffer.push(&mut self.string_pool,text);
    }

    pub fn set_graph_value(&mut self,channel: GraphChannel,value: u8) {
        let graph_buffer = self.graph_buffers.get_mut(channel);
        graph_buffer.push(value);
    }

    pub fn set_label_text(&mut self,channel: LabelChannel,value: &str) {
        let label = self.labels.get_mut(channel);
        label.clear();
        label.push_str(value);
    }

    pub fn set_label_text_fmt<'a>(&'a mut self,channel: LabelChannel,args: fmt::Arguments){
        let label = self.labels.get_mut(channel);
        label.clear();
        let _ = label.write_fmt(args);
    }

    pub fn clear_label(&mut self,channel: LabelChannel) {
        let label = self.labels.get_mut(channel);
        label.clear();
    }

    pub fn render<TFrame>(&self,render_pass: &mut RenderPassBuilder<'_,TFrame>)
    where
        TFrame: MutableFrame
    {
        let size = render_pass.frame().size();

        match &self.render_config.top_left {
            PaneLayout::None => {},
            PaneLayout::One { items } => {
                match &items[0] {
                    PaneItem::None => todo!(),
                    PaneItem::Graph { channel, width } => todo!(),
                    PaneItem::Log { channel} => todo!(),
                    PaneItem::Label { channel } => {
                        let mut text_pass = render_pass.set_pipeline_text();
                        let label = self.labels.get(*channel);
                        text_pass.draw_text::<FontMonoElf>(label,TextRenderConfig {
                            position: [(size.x - 5) as f32,5.0].into(),
                            scale: 2.0,
                            color: WimpyColor::WHITE,
                            line_height: 1.0,
                            word_seperator: ' ',
                            behavior: TextRenderBehavior::RTL,
                        });
                    },
                }
            },
            PaneLayout::Two { items } => todo!(),
            PaneLayout::Three { items } => todo!(),
            PaneLayout::Four { items } => todo!(),
        }
    }
}
