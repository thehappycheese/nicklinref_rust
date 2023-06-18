use super::{
    super::cached::{
        Layer,
        Feature,
        Attributes,
        Cwy,
        Vector2
    },
    IndexedData,
};

impl IndexedData {
    pub fn test_dataset()->Self {
        let data = Layer {
            features: vec![
                Feature {
                    attributes: Attributes {
                        ROAD: "TEST_ROAD".to_owned(),
                        CWY: Cwy::Left,
                        START_SLK: 0.0,
                        END_SLK: 2.0,
                    },
                    geometry: vec![
                        Vector2 { x: 0.0, y: 0.0 },
                        Vector2 { x: 1.0, y: 0.0 },
                        Vector2 { x: 1.0, y: 1.0 },
                    ],
                },
                Feature {
                    attributes: Attributes {
                        ROAD: "TEST_ROAD".to_owned(),
                        CWY: Cwy::Right,
                        START_SLK: 2.0,
                        END_SLK: 4.0,
                    },
                    geometry: vec![
                        Vector2 { x: 1.0, y: 1.0 },
                        Vector2 { x: 1.0, y: 2.0 },
                        Vector2 { x: 2.0, y: 2.0 },
                    ],
                },
            ],
        };
        let index = Self::index_data(&data).unwrap();
        Self{
            data,
            index,
        }

    }
}
