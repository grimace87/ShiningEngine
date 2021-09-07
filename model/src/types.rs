
/// Model struct
/// Represents a model with a name, along with a set of vertices of a generic sized type.
pub struct Model<E> where E : Sized {
    pub name: String,
    pub vertices: Vec<E>
}

impl<E> Model<E> {

    /// Construct a new instance from a set of vertices
    pub fn new_from_components(name: String, vertices: Vec<E>) -> Model<E> {
        Model {
            name,
            vertices
        }
    }

    /// Merge a set of models into a new model under a new name
    pub fn merge(name: &String, source_models: Vec<Model<E>>) -> Model<E> {
        let mut all_vertices = vec![];
        for model in source_models.into_iter() {
            for vertex in model.vertices.into_iter() {
                all_vertices.push(vertex);
            }
        }
        Model {
            name: name.clone(),
            vertices: all_vertices
        }
    }
}

/// StaticVertex struct
/// Vertex definition for a three-dimensional vertex with a position, normal and two-
/// dimensional texture coordinate
#[repr(C)]
#[derive(Copy, Clone)]
pub struct StaticVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub nx: f32,
    pub ny: f32,
    pub nz: f32,
    pub u: f32,
    pub v: f32
}

impl StaticVertex {

    /// Construct a new instance from individual components
    pub fn from_components(
        x: f32,
        y: f32,
        z: f32,
        nx: f32,
        ny: f32,
        nz: f32,
        u: f32,
        v: f32
    ) -> StaticVertex {
        StaticVertex { x, y, z, nx, ny, nz, u, v }
    }
}

impl Default for StaticVertex {

    /// Construct a new instance with position at the origin, texture coordinates at the origin,
    /// and a normal vector pointing in the positive Z direction.
    fn default() -> Self {
        StaticVertex { x: 0.0, y: 0.0, z: 0.0, nx: 0.0, ny: 0.0, nz: 1.0, u: 0.0, v: 0.0 }
    }
}
