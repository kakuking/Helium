use std::io::Read;

use crate::reader::{HeliumError, helpers::Helper};

#[derive(Debug)]
pub enum Transform {
    Affine {
        matrix: [[f64; 4]; 4],
    },

    Scale {
        map_type: String,
        scale: [f64; 3],
        voxel_size: [f64; 3],
        inverse_scale: [f64; 3],
        inverse_scale_squared: [f64; 3],
        inverse_twice_scale: [f64; 3],
    },

    Translation {
        translation: [f64; 3],
    },

    ScaleTranslate {
        map_type: String,
        translation: [f64; 3],
        scale: [f64; 3],
        voxel_size: [f64; 3],
        inverse_scale: [f64; 3],
        inverse_scale_squared: [f64; 3],
        inverse_twice_scale: [f64; 3],
    },

    Unitary {
        matrix: [[f64; 4]; 4],
    },
}

impl Transform {
    pub fn read<R: Read>(
        reader: &mut R,
    ) -> Result<Transform, HeliumError> {
        let map_type = Helper::read_string(reader)?;

        match map_type.as_str() {
            "AffineMap" => {
                let matrix = Helper::read_mat4d(reader)?;

                Ok(Transform::Affine {
                    matrix,
                })
            }

            "UnitaryMap" => {
                let matrix = Helper::read_mat4d(reader)?;

                Ok(Transform::Unitary {
                    matrix,
                })
            }

            "ScaleMap" | "UniformScaleMap" => {
                let scale = Helper::read_vec3d(reader)?;
                let voxel_size = Helper::read_vec3d(reader)?;
                let inverse_scale = Helper::read_vec3d(reader)?;
                let inverse_scale_squared = Helper::read_vec3d(reader)?;
                let inverse_twice_scale = Helper::read_vec3d(reader)?;

                Ok(Transform::Scale {
                    map_type,
                    scale,
                    voxel_size,
                    inverse_scale,
                    inverse_scale_squared,
                    inverse_twice_scale,
                })
            }

            "TranslationMap" => {
                let translation = Helper::read_vec3d(reader)?;

                Ok(Transform::Translation {
                    translation,
                })
            }

            "ScaleTranslateMap"
            | "UniformScaleTranslateMap" => {
                let translation = Helper::read_vec3d(reader)?;
                let scale = Helper::read_vec3d(reader)?;
                let voxel_size = Helper::read_vec3d(reader)?;
                let inverse_scale = Helper::read_vec3d(reader)?;
                let inverse_scale_squared = Helper::read_vec3d(reader)?;
                let inverse_twice_scale = Helper::read_vec3d(reader)?;

                Ok(Transform::ScaleTranslate {
                    map_type,
                    translation,
                    scale,
                    voxel_size,
                    inverse_scale,
                    inverse_scale_squared,
                    inverse_twice_scale,
                })
            }

            other => Err(
                HeliumError::UnsupportedTransform(
                    other.to_owned(),
                )
            ),
        }
    }
}