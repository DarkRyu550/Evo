use crate::settings::Group;
use std::ops::Range;
use std::convert::TryInto;

/** Create a new population from the given preference group. */
pub fn population(group: &Group) -> Vec<Individual> {
	let init2 = ||
		if group.init_to_random {
			[ rand::random(), rand::random() ]
		} else {
			[ 0.0, 0.0 ]
		};
	let init11 = ||
		if group.init_to_random {
			[
				rand::random(), rand::random(), rand::random(), rand::random(),
				rand::random(), rand::random(), rand::random(), rand::random(),
				rand::random(), rand::random(), rand::random(),
			]
		} else {
			[
				0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
				0.0, 0.0, 0.0, 0.0, 0.0,
			]
		};
	let init5 = ||
		if group.init_to_random {
			[
				rand::random(), rand::random(), rand::random(),
				rand::random(), rand::random(),
			]
		} else {
			[
				0.0, 0.0, 0.0, 0.0, 0.0
			]
		};

	(0..group.budget)
		.into_iter()
		.map(|_| {
			Individual {
				position: init2(),
				velocity: init2(),
				energy: init2()[0],
				weights: [
					init11(), init11(), init11(),
					init11(), init11(),
				],
				biases: init5()
			}
		})
		.collect()
}

/** Create a new population with the given parameters and serialize it to
 * `std430`, storing it in the given buffer, after a clear operation. */
pub fn population_bytes_with_buffer(group: &Group, mut buf: Vec<u8>) -> Vec<u8> {
	buf.clear();
	{
		let needed = group.budget as usize * Individual::BYTE_SIZE;
		if buf.capacity() < needed {
			buf.reserve_exact(needed - buf.capacity());
		}
	}
	population(group)
		.into_iter()
		.fold(buf, |mut data, individual| {
			individual.bytes(&mut data);
			data
		})
}

/** Create a new population with the given parameters and serialize it to
 * `std430`, storing it in a newly allocated buffer. */
pub fn population_bytes(group: &Group) -> Vec<u8> {
	population_bytes_with_buffer(group, Vec::with_capacity(group.budget as usize * Individual::BYTE_SIZE))
}

/** Matrix type. */
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4([f32; 16]);
impl Matrix4 {
	pub fn identity() -> Self { Self([
		1.0, 0.0, 0.0, 0.0,
		0.0, 1.0, 0.0, 0.0,
		0.0, 0.0, 1.0, 0.0,
		0.0, 0.0, 0.0, 1.0,
	])}

	pub fn translate(x: f32, y: f32, z: f32) -> Self { Self([
		1.0, 0.0, 0.0,   x,
		0.0, 1.0, 0.0,   y,
		0.0, 0.0, 1.0,   z,
		0.0, 0.0, 0.0, 1.0,
	])}

	pub fn scale(x: f32, y: f32, z: f32) -> Self { Self([
		x, 0.0, 0.0, 0.0,
		0.0,   y, 0.0, 0.0,
		0.0, 0.0,   z, 0.0,
		0.0, 0.0, 0.0, 1.0,
	])}

	pub fn viewport2d(l: f32, r: f32, t: f32, b: f32) -> Self { Self([
		(r - l) / 2.0,           0.0, 0.0, (r + l) / 2.0,
		0.0, (b - t) / 2.0, 0.0, (t + b) / 2.0,
		0.0,           0.0, 1.0,           0.0,
		0.0,           0.0, 0.0,           1.0
	])}

	pub fn ortho2d(l: f32, r: f32, t: f32, b: f32) -> Self { Self([
		2.0 / (r - l),           0.0, 0.0, - ((r + l) / (r - l)),
		0.0, 2.0 / (b - t), 0.0, - ((b + t) / (b - t)),
		0.0,           0.0, 1.0,                   0.0,
		0.0,           0.0, 0.0,                   1.0
	])}

	pub fn perspective(n: f32, f: f32) -> Self {
		Self([
			1.0, 0.0, 0.0,           0.0,
			0.0, 1.0, 0.0,           0.0,
			0.0, 0.0, 1.0 / (f - n), 0.0,
			0.0, 0.0, 1.0,           0.0
		])
	}

	pub fn into_inner(self) -> [f32; 16] { self.0 }

	pub fn transpose(mut self) -> Self {
		let a = |i: usize, j: usize| i * 4 + j;

		for i in 0..4 {
			for j in 0..i {
				let x = self.0[a(i, j)];
				let y = self.0[a(j, i)];

				self.0[a(i, j)] = y;
				self.0[a(j, i)] = x;
			}
		}

		self
	}

	/** Write out the bytes of this structure into a vector.
	 *
	 * # Layout
	 * The bytes written are guaranteed to be laid out in `std430`, as specified
	 * in the OpenGL 4.5 Specification that can be found at
	 * https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf, under
	 * Section 7.6.2.2 (Standard Uniform Block Layout).
	 */
	pub fn bytes(&self, buf: &mut Vec<u8>) -> usize {
		let mut written = 0;
		for val in &self.0 {
			written += write_vec(buf, [*val]);
		}

		written
	}
}
impl std::ops::Mul for Matrix4 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let a = |i: usize, j: usize| self.0[i * 4 + j];
		let b = |i: usize, j: usize| rhs.0[i * 4 + j];

		Self([
			(a(0, 0) * b(0, 0)) + (a(0, 1) * b(1, 0)) + (a(0, 2) * b(2, 0)) + (a(0, 3) * b(3, 0)),
			(a(0, 0) * b(0, 1)) + (a(0, 1) * b(1, 1)) + (a(0, 2) * b(2, 1)) + (a(0, 3) * b(3, 1)),
			(a(0, 0) * b(0, 2)) + (a(0, 1) * b(1, 2)) + (a(0, 2) * b(2, 2)) + (a(0, 3) * b(3, 2)),
			(a(0, 0) * b(0, 3)) + (a(0, 1) * b(1, 3)) + (a(0, 2) * b(2, 3)) + (a(0, 3) * b(3, 3)),
			(a(1, 0) * b(0, 0)) + (a(1, 1) * b(1, 0)) + (a(1, 2) * b(2, 0)) + (a(1, 3) * b(3, 0)),
			(a(1, 0) * b(0, 1)) + (a(1, 1) * b(1, 1)) + (a(1, 2) * b(2, 1)) + (a(1, 3) * b(3, 1)),
			(a(1, 0) * b(0, 2)) + (a(1, 1) * b(1, 2)) + (a(1, 2) * b(2, 2)) + (a(1, 3) * b(3, 2)),
			(a(1, 0) * b(0, 3)) + (a(1, 1) * b(1, 3)) + (a(1, 2) * b(2, 3)) + (a(1, 3) * b(3, 3)),
			(a(2, 0) * b(0, 0)) + (a(2, 1) * b(1, 0)) + (a(2, 2) * b(2, 0)) + (a(2, 3) * b(3, 0)),
			(a(2, 0) * b(0, 1)) + (a(2, 1) * b(1, 1)) + (a(2, 2) * b(2, 1)) + (a(2, 3) * b(3, 1)),
			(a(2, 0) * b(0, 2)) + (a(2, 1) * b(1, 2)) + (a(2, 2) * b(2, 2)) + (a(2, 3) * b(3, 2)),
			(a(2, 0) * b(0, 3)) + (a(2, 1) * b(1, 3)) + (a(2, 2) * b(2, 3)) + (a(2, 3) * b(3, 3)),
			(a(3, 0) * b(3, 0)) + (a(3, 1) * b(1, 0)) + (a(3, 2) * b(2, 0)) + (a(3, 3) * b(3, 0)),
			(a(3, 0) * b(3, 1)) + (a(3, 1) * b(1, 1)) + (a(3, 2) * b(2, 1)) + (a(3, 3) * b(3, 1)),
			(a(3, 0) * b(3, 2)) + (a(3, 1) * b(1, 2)) + (a(3, 2) * b(2, 2)) + (a(3, 3) * b(3, 2)),
			(a(3, 0) * b(3, 3)) + (a(3, 1) * b(1, 3)) + (a(3, 2) * b(2, 3)) + (a(3, 3) * b(3, 3)),
		])
	}
}

/** Set of rendering parameters for the graphics side. */
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RenderParameters {
	/** Transformation matrix from model space into world space. */
	pub world_transformation: Matrix4,
	/** Transformation matrix from world space into projected view space. */
	pub projection: Matrix4,
}
impl RenderParameters {
	/** Write out the bytes of this structure into a vector.
	 *
	 * # Layout
	 * The bytes written are guaranteed to be laid out in `std430`, as specified
	 * in the OpenGL 4.5 Specification that can be found at
	 * https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf, under
	 * Section 7.6.2.2 (Standard Uniform Block Layout).
	 */
	pub fn bytes(&self, buf: &mut Vec<u8>) -> usize {
		let mut written = 0;
		written += self.world_transformation.bytes(buf);
		written += self.projection.bytes(buf);

		written
	}
}

/** Set of compute parameters. */
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ComputeParameters {
	/** Time in seconds since the last iteration. */
	pub delta: f32,
	/** Radius of vision of individuals in the herbivore group. */
	pub herbivore_view_radius: f32,
	/** Radius of vision of individuals in the predator group. */
	pub predator_view_radius: f32,
	/** Maximum speed of a herbivore, in distance per second. */
	pub herbivore_max_speed: f32,
	/** Maximum speed of a predator, in distance per second. */
	pub predator_max_speed: f32,
	/** Penalty for existing and walking as a herbivore. The penalty value
	 * will linearly scale from the first to the second point of this vector as
	 * the walking speed increases from zero to one. */
	pub herbivore_penalty: [f32; 2],
	/** Penalty for existing and walking as a herbivore. The penalty value
	 * will linearly scale from the first to the second point of this vector as
	 * the walking speed increases from zero to one. */
	pub predator_penalty: [f32; 2],
	/** Size of the simulation field. */
	pub simulation: [f32; 2]
}
impl ComputeParameters {
	/** Write out the bytes of this structure into a vector.
	 *
	 * # Layout
	 * The bytes written are guaranteed to be laid out in `std430`, as specified
	 * in the OpenGL 4.5 Specification that can be found at
	 * https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf, under
	 * Section 7.6.2.2 (Standard Uniform Block Layout).
	 */
	pub fn bytes(&self, buf: &mut Vec<u8>) -> usize {
		let mut written = 0;
		written += write_vec(buf, [
			self.delta,
			self.herbivore_view_radius,
			self.predator_view_radius,
			self.herbivore_max_speed,
			self.predator_max_speed
		]);
		written += write_pad(buf, 4);

		written += write_vec(buf, self.herbivore_penalty);
		written += write_vec(buf, self.predator_penalty);
		written += write_vec(buf, self.simulation);

		written
	}
}

/** Back-channel the shader pipelines uses to communicate information back to
 * the host device. */
#[derive(Debug, Clone, PartialEq)]
pub struct BackChannel {
	/** Herbivore dispatch range for the next iteration. */
	pub herbivores: Range<u32>,
	/** Predator dispatch range for the next iteration. */
	pub predators: Range<u32>
}
impl BackChannel {
	/** Create this structure with arbitrary bytes from a buffer. */
	pub fn from_bytes<A: AsRef<[u8]>>(bytes: A) -> Self {
		let data = bytes.as_ref();

		let a = u32::from_ne_bytes((&data[ 0..4 ]).try_into().unwrap());
		let b = u32::from_ne_bytes((&data[ 4..8 ]).try_into().unwrap());
		let c = u32::from_ne_bytes((&data[ 8..12]).try_into().unwrap());
		let d = u32::from_ne_bytes((&data[12..16]).try_into().unwrap());

		if a > b { panic!("lower herbivore bound > upper herbivore bound"); }
		if c > d { panic!("lower predator bound > upper predator bound"); }

		Self {
			herbivores: a..b,
			predators: c..d
		}
	}

	/** Write out the bytes of this structure into a vector.
	 *
	 * # Layout
	 * The bytes written are guaranteed to be laid out in `std430`, as specified
	 * in the OpenGL 4.5 Specification that can be found at
	 * https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf, under
	 * Section 7.6.2.2 (Standard Uniform Block Layout).
	 */
	pub fn bytes(&self, buf: &mut Vec<u8>) -> usize {
		let mut written = 0;
		written += write_u32(buf, self.herbivores.start);
		written += write_u32(buf, self.herbivores.end);

		written += write_u32(buf, self.predators.start);
		written += write_u32(buf, self.predators.end);

		written
	}
}

/** The data for an individual.
 * The input parameters for the individual are the following, in order:
 * - `0`:  Velocity X
 * - `1`:  Velocity Y
 * - `2`:  Red Gradient X
 * - `3`:  Red Gradient Y
 * - `4`:  Red Intensity
 * - `5`:  Green Gradient X
 * - `6`:  Green Gradient Y
 * - `7`:  Green Intensity
 * - `8`:  Blue Gradient X
 * - `9`:  Blue Gradient Y
 * - `10`: Blue Intensity
 *
 * The output parameters of the individual are the following, in order:
 * - `0`: Movement Angle ([0; 1[)
 * - `1`: Movement Speed
 * - `2`: Red Deposit
 * - `3`: Green Deposit
 * - `4`: Blue Deposit
 */
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Individual {
	/** Two-dimensional position vector on the simulation plane. */
	pub position: [f32; 2],

	/** Two-dimensional vector for the direction of the last move. */
	pub velocity: [f32; 2],

	/** Amount of energy this individual can still spend. */
	pub energy: f32,

	/** Weight matrix. This matrix is laid out such that a value at `[i][j]`
	 * means the weight neuron `a[j]` will have on neuron `b[i]`, where `a` is
	 * the input layer and `b` is the output layer. */
	pub weights: [[f32; 11]; 5],

	/** Array of output biases. */
	pub biases: [f32; 5]
}
impl Individual {
	const BYTE_SIZE: usize = 0
	    + 8       		/* position */
		+ 8      		/* velocity */
		+ 4       		/* energy */
	    + 12      		/* pad */
		+ 32      		/* biases */
		+ 64 * 4 * 4	/* weights */
		+ 0;     		/* done */

	/** Write out the bytes of this structure into a vector.
	 *
	 * # Layout
	 * The bytes written are guaranteed to be laid out in `std430`, as specified
	 * in the OpenGL 4.5 Specification that can be found at
	 * https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf, under
	 * Section 7.6.2.2 (Standard Uniform Block Layout).
	 */
	pub fn bytes(&self, buf: &mut Vec<u8>) {
		let mut written = 0;
		written += write_vec(buf, self.position);
		written += write_vec(buf, self.velocity);
		written += write_vec(buf, [self.energy]);

		/* Offset 5N: Pad to the next 4N alignment. */
		written += write_pad(buf, 12);

		/* Offset 8N: Write the weight vectors. */
		written += write_vec(buf, &self.biases[0..4]);
		written += write_vec(buf, &[self.biases[4], 0.0, 0.0, 0.0]);

		/* Offset 16N: Write the A[0-3,0-3] sub-matrices. */
		for i in 0..=3 {
			for j in 0..=3 {
				let column = |i: usize| -> [f32; 4] {
					match j {
						0 => [
							self.weights[i][0],
							self.weights[i][1],
							self.weights[i][2],
							self.weights[i][3],
						],
						1 => [
							self.weights[i][4],
							self.weights[i][5],
							self.weights[i][6],
							self.weights[i][7],
						],
						2 => [
							self.weights[i][8],
							self.weights[i][9],
							self.weights[i][10],
							0.0,
						],
						3 => [0.0, 0.0, 0.0, 0.0],
						_ => unreachable!()
					}
				};
				let row = |i: usize| match i {
					i @ _ if i < 5  => column(i),
					i @ _ if i < 16 => [0.0, 0.0, 0.0, 0.0],
					_ => unreachable!()
				};

				written += write_vec(buf, &row(i * 4 + 0));
				written += write_vec(buf, &row(i * 4 + 1));
				written += write_vec(buf, &row(i * 4 + 2));
				written += write_vec(buf, &row(i * 4 + 3));
			}
		}

		debug_assert_eq!(written, Self::BYTE_SIZE, "Wrong byte size after write, please update Individual::BYTE_SIZE");
	}
}

/** Writes the given number of zero bytes into the buffer. */
fn write_pad(buf: &mut Vec<u8>, size: usize) -> usize {
	let new = buf.len().checked_add(size)
		.expect("WHY DO YOU NEED THIS MANY BYTES FOR PADDING?");
	buf.resize(new, 0);

	size
}

/** Writes the given number into the buffer. */
fn write_u32(buf: &mut Vec<u8>, val: u32) -> usize {
	let data = val.to_ne_bytes();
	buf.extend_from_slice(&data[..]);

	data.len()
}

/** Writes the given float vector into the buffer. */
fn write_vec<A>(buf: &mut Vec<u8>, dat: A) -> usize
	where A: AsRef<[f32]> {

	let data = dat.as_ref();
	(0..data.len()).into_iter()
		.map(|i| data[i])
		.map(|p| f32::to_ne_bytes(p))
		.map(|bytes| {
			buf.extend_from_slice(&bytes[..]);
			bytes.len()
		})
		.sum()
}
