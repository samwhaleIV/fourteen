//TODO: Groups are redundant. Make elements nestable.

use std::{
    collections::{
        HashMap,
        HashSet, VecDeque
    },
    time::{
        Duration,
        Instant
    }
};

use generational_arena::{
    Arena,
    Index
};

use crate::{
    shared::{
        Area,
        Color,
        Layout
    }, wgpu::Frame
};

enum AnimationEffect {
    FinalScale,
    FadeIn,
    FadeOut
}

struct Animation {
    start: Instant,
    duration: Duration,
    effect: AnimationEffect
}

impl Default for AnimatedLayout {
    fn default() -> Self {
        return Self {
            value: Layout::default(),
            pending_value: None,
            start: Instant::now(),
            duration: Duration::ZERO,
        }
    }
}

struct GroupInternal {
    children: HashSet<Group>,
    elements: HashSet<Element>,
    layout: AnimatedLayout
}

impl Default for GroupInternal {
    fn default() -> Self {
        return Self {
            children: Default::default(),
            elements: Default::default(),
            layout: AnimatedLayout::default()
        }
    }
}

struct AnimatedLayout {
    value: Layout,
    pending_value: Option<Layout>,
    start: Instant,
    duration: Duration
}

pub enum InteractionState {
    None,
    Disabled,
    Enabled
}

struct ElementInternal {
    style: ElementStyle,
    layout: AnimatedLayout,
    interaction: InteractionState,
    focus_animation: Option<Animation>,
    capture_animation: Option<Animation>
}

pub struct ElementStyle {
    texture: Texture,
    uv: Area,
    rotation: f32,
    color: Color,
}

#[derive(Clone,Copy,Hash,PartialEq,Eq)]
pub struct Group {
    index: Index
}

#[derive(Clone,Copy,Hash,PartialEq,Eq)]
pub struct Element {
    index: Index
}

#[derive(Clone,Copy)]
pub struct Texture {
    index: Index
}

pub struct Page<TTexture> {
    groups: Arena<GroupInternal>,
    elements: Arena<ElementInternal>,

    element_parent_table: HashMap<Element,Group>,
    group_parent_table: HashMap<Group,Group>,

    top_level_group: Group,

    focus_element: Option<Element>,
    captured_element: Option<Element>,

    texture_binds: Arena<TTexture>,

    //output_buffer: Vec<(DrawData,TTexture)>
}

pub enum PageEvent {
    ClickStart(Element),
    ClickRelease(Element),
    FocusStart(Element),
    FocusEnd(Element),
    FocusLost
}

pub enum PageActionResult {
    Success,
    NullReferenceElement,
    NullReferenceGroup,
    NullReferenceGroupDestination,
    NullReferenceGroupSource,
    ElementAlreadyInGroup,
    GroupAlreadyInGroup,
    CircularGroupReference
}

impl<TTexture> Page<TTexture> {
    pub fn create() -> Self {
        let mut groups = Arena::new();
        let top_level_group = Group { index: groups.insert(GroupInternal::default()) };

        return Self {
            groups, top_level_group, focus_element: None, captured_element: None,
            elements: Default::default(), texture_binds: Default::default(),
            element_parent_table: Default::default(), group_parent_table: Default::default(), 
        }
    }

    pub fn create_element(&mut self,layout: Layout,interaction: InteractionState,style: ElementStyle) -> Element {
        return Element {
            index: self.elements.insert(ElementInternal {
                style,
                interaction,
                layout: AnimatedLayout { value: layout, ..Default::default() },
                focus_animation: None,
                capture_animation: None,
            })
        };
    }

    pub fn create_group(&mut self,layout: Layout) -> Group {
        return Group {
            index: self.groups.insert(GroupInternal {
                layout: AnimatedLayout { value: layout, ..Default::default() }, ..Default::default()
            })
        }
    }

    pub fn bind_texture(&mut self,texture: TTexture) -> Texture {
        return Texture {
            index: self.texture_binds.insert(texture)
        }
    }

    pub fn add_element_to_group(&mut self,child: Element,parent: Group) -> PageActionResult {
        if !self.elements.contains(child.index) {
            return PageActionResult::NullReferenceElement;
        };
        if self.element_parent_table.contains_key(&child) {
            return PageActionResult::ElementAlreadyInGroup;
        }
        let Some(destination) = self.groups.get_mut(parent.index) else {
            return PageActionResult::NullReferenceGroupDestination;
        };
        destination.elements.insert(child);
        self.element_parent_table.insert(child,parent);
        return PageActionResult::Success;
    }

    pub fn add_group_to_group(&mut self,child: Group,parent: Group) -> PageActionResult {
        if child.index == parent.index {
            return PageActionResult::CircularGroupReference;
        }
        if !self.groups.contains(child.index) {
            return PageActionResult::NullReferenceGroupSource;
        }
        if self.group_parent_table.contains_key(&child) {
            return PageActionResult::GroupAlreadyInGroup;
        }
        let Some(destination) = self.groups.get_mut(parent.index) else {
            return PageActionResult::NullReferenceGroupDestination;
        };
        destination.children.insert(child);
        self.group_parent_table.insert(child,parent);

        return PageActionResult::Success;
    }

    pub fn delete_element(&mut self,element: Element) -> PageActionResult {
        if self.elements.remove(element.index).is_none() {
            return PageActionResult::NullReferenceElement;
        }
        let Some(parent_reference) = self.element_parent_table.remove(&element) else {
            return PageActionResult::Success;
        };
        if let Some(parent) = self.groups.get_mut(parent_reference.index) {
            parent.elements.remove(&element);
        };
        return PageActionResult::Success;
    }

    pub fn delete_group(&mut self,group: Group) -> PageActionResult {

        if !self.groups.contains(group.index) {
            return PageActionResult::NullReferenceGroup;
        }

        let mut deletion_queue = VecDeque::from([group]);

        while let Some(group_reference) = deletion_queue.pop_back() {
            self.group_parent_table.remove(&group_reference);
            let Some(group) = self.groups.remove(group_reference.index) else {
                continue;
            };
            for child_element in group.elements {
                self.elements.remove(child_element.index);
                self.element_parent_table.remove(&child_element);
            }
            for child_group in group.children {
                deletion_queue.push_back(child_group);
            }
        }
        return PageActionResult::Success;
    }

    pub fn add_element_to_page(&mut self,child: Element) -> PageActionResult {
        return self.add_element_to_group(child,self.top_level_group);
    }

    pub fn add_group_to_page(&mut self,child: Group) -> PageActionResult {
        return self.add_group_to_group(child,self.top_level_group);
    }
 
    pub fn update_element_style(&mut self,element: Element,style: ElementStyle) -> PageActionResult {
        let Some(element) = self.elements.get_mut(element.index) else {
            return PageActionResult::NullReferenceElement;
        };
        element.style = style;
        return PageActionResult::Success;
    }
   
    pub fn update_element_interaction(&mut self,element: Element,interaction: InteractionState) -> PageActionResult {
            let Some(element) = self.elements.get_mut(element.index) else {
            return PageActionResult::NullReferenceElement;
        };
        element.interaction = interaction;
        return PageActionResult::Success;    
    }

    pub fn update_group_layout(&mut self,group: Group,layout: Layout) -> PageActionResult {
        let Some(group) = self.groups.get_mut(group.index) else {
            return PageActionResult::NullReferenceElement;
        };
        group.layout = AnimatedLayout { value: layout, ..Default::default() };
        return PageActionResult::Success;
    }

    pub fn update_element_layout(&mut self,element: Element,layout: Layout) -> PageActionResult {
        let Some(element) = self.elements.get_mut(element.index) else {
            return PageActionResult::NullReferenceElement;
        };
        /* Drops animation already happening. TODO: Change that. Freeze any current animation and shift the start point. */
        element.layout = AnimatedLayout { value: layout, ..Default::default() };
        return PageActionResult::Success;
    }

    pub fn update(&mut self,mouse_pressed: bool,mouse_position: (f32,f32),page_size: (u32,u32)) -> &Vec<PageEvent> {
        todo!();
    }

    pub fn render_to_frame(&self,frame: Frame) {
        
    }
}
