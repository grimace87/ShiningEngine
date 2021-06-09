
use model::factory::StaticVertex;

const SEMANTIC_VERTEX: &str = "VERTEX";
const SEMANTIC_POSITION: &str = "POSITION";
const SEMANTIC_NORMAL: &str = "NORMAL";
const SEMANTIC_TEX_COORD: &str = "TEXCOORD";

#[derive(Debug, Deserialize)]
pub struct GeometryLibrary {
    #[serde(rename = "geometry", default)]
    pub items: Vec<Geometry>
}

#[derive(Debug, Deserialize)]
pub struct Geometry {
    pub id: String,
    pub name: String,
    pub mesh: Mesh
}

#[derive(Debug, Deserialize)]
pub struct Mesh {
    vertices: Vertices,
    triangles: Triangles,

    #[serde(rename = "source", default)]
    sources: Vec<Source>
}

impl Mesh {
    pub fn get_vertex_data(&self) -> Vec<StaticVertex> {
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
pub struct VisualScenesLibrary {
    pub visual_scene: VisualScene
}

#[derive(Debug, Deserialize)]
pub struct VisualScene {
    id: String,
    name: String,

    #[serde(rename = "node", default)]
    pub nodes: Vec<Node>
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,

    #[serde(rename = "type")]
    pub node_type: String,

    pub matrix: Matrix,

    #[serde(default)]
    pub instance_geometry: Option<Instance>,

    #[serde(default)]
    pub instance_camera: Option<Instance>,

    #[serde(default)]
    pub instance_light: Option<Instance>
}

#[derive(Debug, Deserialize)]
pub struct Matrix {
    sid: String,

    #[serde(rename = "$value", default)]
    values: String
}

impl Matrix {
    pub fn decode_element_data(&self) -> Vec<f32> {
        let numbers: Result<Vec<f32>, _> = self.values.split(' ')
            .map(str::parse)
            .collect();
        numbers.expect("Failed to parse float array for matrix")
    }
}

#[derive(Debug, Deserialize)]
pub struct Instance {
    pub url: String
}
