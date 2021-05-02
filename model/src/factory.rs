
use std::{
    path::Path,
    fs::File,
    io::Write
};

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

pub const VERTEX_SIZE_BYTES: usize = 32;
pub const VERTEX_FLOAT_COUNT: usize = 8;

impl StaticVertex {
    pub fn from_components(x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32, u: f32, v: f32) -> StaticVertex {
        StaticVertex { x, y, z, nx, ny, nz, u, v }
    }
}

impl Default for StaticVertex {
    fn default() -> Self {
        StaticVertex { x: 0.0, y: 0.0, z: 0.0, nx: 0.0, ny: 0.0, nz: 1.0, u: 0.0, v: 0.0 }
    }
}

pub struct Model {
    pub name: String,
    pub vertices: Vec<StaticVertex>
}

impl Model {

    pub fn new_from_components(name: String, vertices: Vec<StaticVertex>) -> Model {
        Model {
            name,
            vertices
        }
    }

    pub unsafe fn new_from_bytes(bytes: &[u8]) -> Result<Model, String> {

        // Read in vertex data
        let name_length: usize = *(bytes as *const [u8] as *const u32) as usize;
        let name = String::from_utf8_unchecked(bytes[4..(4 + name_length)].to_vec());
        let vertex_count: u32 = *(&bytes[(4 + name_length)..(8 + name_length)] as *const [u8] as *const u32);
        let mut vertices: Vec<StaticVertex> = vec![StaticVertex::default(); vertex_count as usize];
        let vertex_src_ptr = bytes[(8 + name_length)..(8 + name_length + vertex_count as usize * VERTEX_SIZE_BYTES)].as_ptr() as *const StaticVertex;
        let vertex_src_slice = std::slice::from_raw_parts(vertex_src_ptr, vertex_count as usize);
        vertices.copy_from_slice(vertex_src_slice);

        // Done
        Ok(Model {
            name,
            vertices
        })
    }

    pub unsafe fn write_to_binary_file(&self, file_path: &Path) -> Result<(), String> {

        // Open the file for writing
        let mut file = File::create(file_path)
            .map_err(|e| format!("Error opening file: {:?} - {:?}", file_path, e))?;

        // Put all the model's data in there
        file.write_all(&(self.name.len() as u32).to_ne_bytes()).unwrap();
        file.write_all(self.name.as_bytes()).unwrap();
        file.write_all(&(self.vertices.len() as u32).to_ne_bytes()).unwrap();
        for vertex in self.vertices.iter() {
            file.write_all(&*(vertex as *const StaticVertex as *const [u8; VERTEX_SIZE_BYTES])).unwrap();
        }

        // Done
        Ok(())
    }
}
