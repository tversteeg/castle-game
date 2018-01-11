pub struct Terrain {
    pub buffer: Vec<u32>,

    width: usize,
    height: usize
}

impl Terrain {
    pub fn new(size: (usize, usize)) -> Self {
        Terrain {
            buffer: vec![0xFFFF00FF; size.0 * size.1],

            width: size.0,
            height: size.1,
        }
    }
}
