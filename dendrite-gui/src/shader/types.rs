use dendrite::graph::Layer;

/// Instance data for rendering a single node.
/// Matches the NodeInstance struct in graph.wgsl.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NodeInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub flags: u32,
    pub _padding: [u32; 3],
}

impl NodeInstance {
    pub fn new(position: [f32; 2], size: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            position,
            size,
            color,
            flags: 0,
            _padding: [0; 3],
        }
    }

    #[allow(dead_code)]
    pub fn set_selected(&mut self, selected: bool) {
        if selected {
            self.flags |= 0x1;
        } else {
            self.flags &= !0x1;
        }
    }

    #[allow(dead_code)]
    pub fn set_hovered(&mut self, hovered: bool) {
        if hovered {
            self.flags |= 0x2;
        } else {
            self.flags &= !0x2;
        }
    }

    #[allow(dead_code)]
    pub fn set_in_cycle(&mut self, in_cycle: bool) {
        if in_cycle {
            self.flags |= 0x4;
        } else {
            self.flags &= !0x4;
        }
    }
}

/// Get the color for a given layer.
pub fn layer_color(layer: Layer) -> [f32; 4] {
    match layer {
        Layer::Entry => [0.8, 0.4, 0.8, 1.0],
        Layer::App => [0.4, 0.8, 0.8, 1.0],
        Layer::Core => [0.4, 0.4, 0.9, 1.0],
        Layer::Platform => [0.4, 0.8, 0.4, 1.0],
        Layer::Driver => [0.9, 0.8, 0.3, 1.0],
        Layer::Arch => [0.9, 0.4, 0.4, 1.0],
        Layer::Unknown => [0.5, 0.5, 0.5, 1.0],
    }
}
