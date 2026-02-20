use std::collections::VecDeque;
use std::marker::PhantomData;

use crate::app::WimpyContext;
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
    fn get(&mut self,channel: TChannel) -> &mut T {
        return &mut self.buffers[channel.into()];
    }
}

type GraphBufferSet = BufferSet<GRAPH_CHANNEL_COUNT,GraphBuffer,GraphChannel>;
#[derive(Default)]
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
#[derive(Default)]
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
#[derive(Default)]
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
        width: GraphWidth
    },
    Log {
        channel: LogChannel
    },
    Label {
        channel: LabelChannel
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

    pub fn log(&mut self,channel: LogChannel,text: &str) {
        let log_buffer = self.log_buffers.get(channel);
        log_buffer.push(&mut self.string_pool,text);
    }

    pub fn set_graph_value(&mut self,channel: GraphChannel,value: u8) {
        let graph_buffer = self.graph_buffers.get(channel);
        graph_buffer.push(value);
    }

    pub fn set_label(&mut self,channel: LabelChannel,value: &str) {
        let label = self.labels.get(channel);
        label.clear();
        label.push_str(value);
    }

    pub fn render(&self,context: &mut WimpyContext) {

    }
}
