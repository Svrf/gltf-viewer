use std::ffi::CStr;
use std::rc::Rc;

use gltf;

use render::math::*;
use render::mesh::Mesh;
use render::scene::Scene;
use shader::Shader;

pub struct Node {
    // TODO!!: camera?
    pub children: Vec<Node>,
    pub matrix: Matrix4,
    pub mesh: Option<Rc<Mesh>>,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub translation: Vector3,
    // TODO: weights
    // weights_id: usize,
    pub name: Option<String>,

    final_transform: Matrix4, // including parent transforms
}

impl Node {
    pub fn from_gltf(g_node: gltf::scene::Node, scene: &mut Scene) -> Node {
        let m = &g_node.matrix();
        let matrix = Matrix4::new(
            m[0], m[1], m[2], m[2],
            m[4], m[5], m[6], m[7],
            m[8], m[9], m[10], m[11],
            m[12], m[13], m[14], m[15],
        );
        let r = &g_node.rotation();
        let rotation = Quaternion::new(r[3], r[0], r[1], r[2]); // NOTE: different element order!
        let mut mesh = None;
        if let Some(g_mesh) = g_node.mesh() {
            if let Some(g_mesh) = scene.meshes.iter().find(|mesh| (***mesh).index == g_mesh.index()) {
                mesh = Some(g_mesh.clone());
            }

            if mesh.is_none() { // not using else due to borrow-checking madness
                mesh = Some(Rc::new(Mesh::from_gltf(g_mesh)));
                scene.meshes.push(mesh.clone().unwrap());
            }
        }
        Node {
            children: g_node.children()
                .map(|g_node| Node::from_gltf(g_node, scene))
                .collect(),
            // TODO: why doesn't this work?
            // matrix: Matrix4::from(&g_node.matrix()),
            matrix: matrix,
            mesh: mesh,
            rotation: rotation,
            scale: Vector3::from(g_node.scale()),
            translation: Vector3::from(g_node.translation()),
            name: g_node.name().map(|s| s.into()),

            final_transform: Matrix4::identity(), // TODO!: init already?
        }
    }

    pub fn draw(&self, shader: &Shader, model_matrix: &Matrix4) {
        // TODO!: handle case of neither TRS nor matrix -> identity (or already works?)
        let mut model_matrix = *model_matrix;
        if !self.matrix.is_identity() { // TODO: optimize - determine in constructor
            model_matrix = model_matrix * self.matrix;
        }
        else {
            // TODO: optimize (do on setup / cache)
            model_matrix = model_matrix *
                Matrix4::from_translation(self.translation) *
                Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) *
                Matrix4::from(self.rotation);
        }

        if let Some(ref mesh) = self.mesh {
            // TODO: assume identity set and don't set if identity here?
            unsafe {
                shader.set_mat4(c_str!("model"), &model_matrix);
            }

            (*mesh).draw(shader);
        }
        for node in &self.children {
            node.draw(shader, &model_matrix);
        }
    }
}
