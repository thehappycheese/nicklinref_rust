use std::ops::Index;
use super::{
    super::cached::Cwy
};


#[allow(non_snake_case)]
pub struct RoadDataByCwy {
	pub Left: Option<(usize, usize)>,
	pub Right: Option<(usize, usize)>,
	pub Single: Option<(usize, usize)>,
}

impl RoadDataByCwy {
	pub fn new(
		l: Option<(usize, usize)>,
		r: Option<(usize, usize)>,
		s: Option<(usize, usize)>,
	) -> Self {
		Self {
			Left: l,
			Right: r,
			Single: s,
		}
	}
	pub fn new_from_cwy(cwy: &Cwy, range: (usize, usize)) -> Self {
		match cwy {
			Cwy::Left => Self::new(Some(range), None, None),
			Cwy::Right => Self::new(None, Some(range), None),
			Cwy::Single => Self::new(None, None, Some(range)),
		}
	}
	pub fn with_updated_cwy(&self, cwy: &Cwy, range: (usize, usize)) -> Self {
		match cwy {
			Cwy::Left => Self::new(Some(range), self.Right, self.Single),
			Cwy::Right => Self::new(self.Left, Some(range), self.Single),
			Cwy::Single => Self::new(self.Left, self.Right, Some(range)),
		}
	}
}

impl Index<&Cwy> for RoadDataByCwy {
	type Output = Option<(usize, usize)>;
	fn index(&self, index: &Cwy) -> &Self::Output {
		match index {
			Cwy::Left => &self.Left,
			Cwy::Right => &self.Right,
			Cwy::Single => &self.Single,
		}
	}
}
