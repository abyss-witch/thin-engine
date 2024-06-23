/// often used for screen effects like fxaa, fog and colour correction.
pub mod screen {
    use glium_types::prelude::*;
    pub const VERTICES: [Vertex; 4] = [
        Vertex::new( 1.0,  1.0, 0.0),
        Vertex::new( 1.0, -1.0, 0.0),
        Vertex::new(-1.0,  1.0, 0.0),
        Vertex::new(-1.0, -1.0, 0.0)
    ];
    pub const UVS: [TextureCoords; 4] = [
        TextureCoords::new(1.0 , 1.0),
        TextureCoords::new(1.0 , 0.0),
        TextureCoords::new(0.0 , 1.0),
        TextureCoords::new(0.0 , 0.0)
    ];
    pub const INDICES: [u32; 6] = [
        0, 2, 1,
        1, 2, 3
    ];
}
pub use glium_types::teapot;
