
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

use model::factory::{Model, StaticVertex};

use serde_xml_rs::from_reader;

const SEMANTIC_VERTEX: &str = "VERTEX";
const SEMANTIC_POSITION: &str = "POSITION";
const SEMANTIC_NORMAL: &str = "NORMAL";
const SEMANTIC_TEX_COORD: &str = "TEXCOORD";

#[derive(Debug, Deserialize)]
pub struct COLLADA {
    library_geometries: GeometryLibrary,
    library_visual_scenes: VisualScenesLibrary
}

impl COLLADA {
    pub fn new(file_data: &[u8]) -> COLLADA {
        from_reader(file_data).unwrap()
    }

    pub fn extract_models(&self) -> Vec<Model> {
        let mut models: Vec<Model> = vec![];
        for geometry in self.library_geometries.items.iter() {
            let mesh = &geometry.mesh;
            let mut vertex_data = mesh.get_vertex_data();
            if let Some(scene_matrix) = self.find_transform_for(&geometry.id) {
                Self::transform_vertices(&mut vertex_data, scene_matrix);
            }
            models.push(Model::new_from_components(String::from(&geometry.name), vertex_data));
        }
        models
    }

    fn find_transform_for(&self, geometry_id: &String) -> Option<&Matrix> {
        let node = self.library_visual_scenes.visual_scene.nodes.iter().find(|n| {
            match n {
                Node {
                    id: _id,
                    name: _name,
                    node_type: _node_type,
                    matrix: _matrix,
                    instance_camera: _instance_camera,
                    instance_light: _instance_light,
                    instance_geometry: Some(i)
                } => &i.url[1..i.url.len()] == geometry_id.as_str(),
                _ => false
            }
        });
        match node {
            Some(n) => Some(&n.matrix),
            None => None
        }
    }

    fn transform_vertices(vertices: &mut Vec<StaticVertex>, matrix: &Matrix) {
        let m = matrix.decode_element_data();
        for vertex in vertices.iter_mut() {

            // Transform positions
            let x = vertex.x;
            let y = vertex.y;
            let z = vertex.z;
            vertex.x = x * m[0] + y * m[1] + z * m[2] + m[3];
            vertex.y = x * m[4] + y * m[5] + z * m[6] + m[7];
            vertex.z = x * m[8] + y * m[9] + z * m[10] + m[11];

            // Transform normals
            let x = vertex.nx;
            let y = vertex.ny;
            let z = vertex.nz;
            vertex.nx = x * m[0] + y * m[1] + z * m[2];
            vertex.ny = x * m[4] + y * m[5] + z * m[6];
            vertex.nz = x * m[8] + y * m[9] + z * m[10];
        }
    }
}

#[derive(Debug, Deserialize)]
struct GeometryLibrary {
    #[serde(rename = "geometry", default)]
    items: Vec<Geometry>
}

#[derive(Debug, Deserialize)]
struct Geometry {
    id: String,
    name: String,
    mesh: Mesh
}

#[derive(Debug, Deserialize)]
struct Mesh {
    vertices: Vertices,
    triangles: Triangles,

    #[serde(rename = "source", default)]
    sources: Vec<Source>
}

impl Mesh {
    fn get_vertex_data(&self) -> Vec<StaticVertex> {
        let interleaved_indices = self.decode_triangle_indices();
        let position_data = self.decode_position_data();
        let normal_data = self.decode_normal_data();
        let tex_coord_data = self.decode_tex_coord_data();

        let mut index = 0;
        let mut vertices = vec![];
        loop {
            if index >= interleaved_indices.len() {
                break;
            }
            let position_index = interleaved_indices[index];
            let normal_index = interleaved_indices[index + 1];
            let tex_coord_index = interleaved_indices[index + 2];
            vertices.push(StaticVertex::from_components(
                position_data[position_index * 3],
                position_data[position_index * 3 + 1],
                position_data[position_index * 3 + 2],
                normal_data[normal_index * 3],
                normal_data[normal_index * 3 + 1],
                normal_data[normal_index * 3 + 2],
                tex_coord_data[tex_coord_index * 2],
                tex_coord_data[tex_coord_index * 2 + 1]
            ));
            index += 3;
        }
        vertices
    }

    fn decode_triangle_indices(&self) -> Vec<usize> {
        let value_string = &self.triangles.polygons.values;
        let numbers: Result<Vec<usize>, _> = value_string.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse integer array for triangles")
    }

    fn decode_position_data(&self) -> Vec<f32> {
        let vertex_input = self.triangles.inputs.iter().find(|input| input.semantic.as_str() == SEMANTIC_VERTEX)
            .expect("No VERTEX input found for triangles");
        if self.vertices.id.as_str() != &vertex_input.source[1..vertex_input.source.len()] {
            panic!("Mesh vertices id does not match triangles vertex input source");
        }
        if self.vertices.input.semantic.as_str() != SEMANTIC_POSITION {
            panic!("Mesh vertices input does not have POSITION semantic");
        }
        let position_source_id = &self.vertices.input.source;
        let position_source_id = &position_source_id[1..position_source_id.len()];
        let position_source = self.sources.iter().find(|source| source.id.as_str() == position_source_id)
            .expect("Did not find position source for mesh");
        if position_source.technique_common.accessor.params.len() != 3 {
            panic!("Position source does not have 3 parameters");
        }
        let value_string = &position_source.float_data.values;
        let numbers: Result<Vec<f32>, _> = value_string.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse float array for position data")
    }

    fn decode_normal_data(&self) -> Vec<f32> {
        let normal_input = self.triangles.inputs.iter().find(|input| input.semantic.as_str() == SEMANTIC_NORMAL)
            .expect("No NORMAL input found for triangles");
        let normal_source_id = &normal_input.source;
        let normal_source_id = &normal_source_id[1..normal_source_id.len()];
        let normal_source = self.sources.iter().find(|source| source.id.as_str() == normal_source_id)
            .expect("Did not find normal source for mesh");
        if normal_source.technique_common.accessor.params.len() != 3 {
            panic!("Normal source does not have 3 parameters");
        }
        let value_string = &normal_source.float_data.values;
        let numbers: Result<Vec<f32>, _> = value_string.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse float array for normal data")
    }

    fn decode_tex_coord_data(&self) -> Vec<f32> {
        let tex_coord_input = self.triangles.inputs.iter().find(|input| input.semantic.as_str() == SEMANTIC_TEX_COORD)
            .expect("No TEXCOORD input found for triangles");
        let tex_coord_source_id = &tex_coord_input.source;
        let tex_coord_source_id = &tex_coord_source_id[1..tex_coord_source_id.len()];
        let tex_coord_source = self.sources.iter().find(|source| source.id.as_str() == tex_coord_source_id)
            .expect("Did not find tex coord source for mesh");
        if tex_coord_source.technique_common.accessor.params.len() != 2 {
            panic!("Tex coord source does not have 2 parameters");
        }
        let value_string = &tex_coord_source.float_data.values;
        let numbers: Result<Vec<f32>, _> = value_string.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse float array for tex coord data")
    }
}

#[derive(Debug, Deserialize)]
struct Vertices {
    id: String,
    input: Input
}

#[derive(Debug, Deserialize)]
struct Input {
    semantic: String,
    source: String,

    #[serde(default)]
    offset: i32
}

#[derive(Debug, Deserialize)]
struct Triangles {
    count: i32,

    #[serde(rename = "input", default)]
    inputs: Vec<Input>,

    #[serde(rename = "p", default)]
    polygons: IntegerArray
}

#[derive(Debug, Deserialize, Default)]
struct IntegerArray {

    #[serde(rename = "$value", default)]
    values: String
}

#[derive(Debug, Deserialize)]
struct Source {
    id: String,
    technique_common: TechniqueCommon,

    #[serde(rename = "float_array", default)]
    float_data: FloatArray
}

#[derive(Debug, Deserialize, Default)]
struct FloatArray {
    id: String,
    count: i32,

    #[serde(rename = "$value", default)]
    values: String
}

#[derive(Debug, Deserialize)]
struct TechniqueCommon {
    accessor: Accessor
}

#[derive(Debug, Deserialize)]
struct Accessor {
    source: String,
    count: i32,
    stride: i32,

    #[serde(rename = "param", default)]
    params: Vec<Param>
}

#[derive(Debug, Deserialize)]
struct Param {
    name: String,

    #[serde(rename = "type", default)]
    param_type: String
}

#[derive(Debug, Deserialize)]
struct VisualScenesLibrary {
    visual_scene: VisualScene
}

#[derive(Debug, Deserialize)]
struct VisualScene {
    id: String,
    name: String,

    #[serde(rename = "node", default)]
    nodes: Vec<Node>
}

#[derive(Debug, Deserialize)]
struct Node {
    id: String,
    name: String,

    #[serde(rename = "type")]
    node_type: String,

    matrix: Matrix,

    #[serde(default)]
    instance_geometry: Option<Instance>,

    #[serde(default)]
    instance_camera: Option<Instance>,

    #[serde(default)]
    instance_light: Option<Instance>
}

#[derive(Debug, Deserialize)]
struct Matrix {
    sid: String,

    #[serde(rename = "$value", default)]
    values: String
}

impl Matrix {
    fn decode_element_data(&self) -> Vec<f32> {
        let numbers: Result<Vec<f32>, _> = self.values.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse float array for matrix")
    }
}

#[derive(Debug, Deserialize)]
struct Instance {
    url: String
}
