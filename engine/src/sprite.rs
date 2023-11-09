use cgmath::{Matrix4, Vector3};

pub fn compute_model_matrix(position: Vector3<f32>) -> Matrix4<f32> {
    let pos_mat = Matrix4::from_translation(position);
    let scale_mat = Matrix4::from_nonuniform_scale(1.0, 0.5, 1.0);
    pos_mat * scale_mat
}
