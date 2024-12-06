pub use glam::{ Vec2 };



#[derive(Copy,Clone)]
pub enum PinMode
{
    Unpinned,
    PinX,
    PinY,
    PinXY,
}

#[derive(Copy,Clone)]
pub struct AnchorPoint
{
    pub p : Vec2,
    pub pin : PinMode,
}

#[derive(Default)]
pub struct ConstraintSystem
{
    pub anchors : Vec<AnchorPoint>,
}

// impl Default for ConstraintSystem {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl ConstraintSystem
{
    pub fn new() -> Self {
        Self { anchors: Vec::new() }
    }

    // todo: way to wrap index so it's typesafe?
    pub fn add_anchor( &mut self, p : Vec2 ) -> usize {
        let index = self.anchors.len();
        self.anchors.push( AnchorPoint { p : p, pin : PinMode::Unpinned });
        index
    }
}









pub fn add_vec2( left : Vec2, right: Vec2 ) -> Vec2 {
    return left + right;
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
