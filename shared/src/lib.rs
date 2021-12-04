use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct BenchStats {
	pub average_ns: i64,
	pub deviation_ns: i64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Bench {
	pub p1: Option<BenchStats>,
	pub p2: Option<BenchStats>,
	pub parse: Option<BenchStats>,
}
