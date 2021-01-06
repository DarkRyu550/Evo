use crate::settings::Group;

/** Create a new population from the given preference group. */
pub fn population(group: &Group) -> Vec<Individual> {
	let init2 = ||
		if group.init_to_random {
			[ rand::random(), rand::random() ]
		} else {
			[ 0.0, 0.0 ]
		};
	let init3 = ||
		if group.init_to_random {
			[ rand::random(), rand::random(), rand::random() ]
		} else {
			[ 0.0, 0.0, 0.0 ]
		};

	(0..group.budget)
		.into_iter()
		.map(|_| {
			Individual {
				position: init2(),
				velocity: init2(),
				red_bias: init3(),
				green_bias: init3(),
				blue_bias: init3(),
				red_weight: init3(),
				green_weight: init3(),
				blue_weight: init3(),
				deposit_bias: [
					group.signature.red   as f32,
					group.signature.green as f32,
					group.signature.blue  as f32,
				],
				movement_bias: if group.init_to_random {
					let angle =
						  rand::random::<f32>()
						* std::f32::consts::PI * 2.0;
					[ angle.cos(), angle.sin() ]
				} else {
					[ 0.0, 0.0 ]
				}
			}
		})
		.collect()
}

/** Create a new population with the given parameters and serialize it to
 * `std430`, storing it in the given buffer, after a clear operation. */
pub fn population_bytes_with_buffer(group: &Group, mut buf: Vec<u8>) -> Vec<u8> {
	buf.clear();
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
	population_bytes_with_buffer(group, Vec::new())
}

/** The data for an individual. */
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Individual {
	/** Two-dimensional position vector on the simulation plane. */
	pub position: [f32; 2],

	/** Two-dimensional vector for the direction of the last move. */
	pub velocity: [f32; 2],

	/** Four-dimensional bias vector for the red chemical.
	 *
	 * The first two dimensions will be applied to the gradient direction of
	 * the chemical in the field of view of the individual. The third dimension
	 * will be applied to the intensity value of the chemical. */
	pub red_bias: [f32; 3],

	/** Four-dimensional bias vector for the green chemical.
	 *
	 * The first two dimensions will be applied to the gradient direction of
	 * the chemical in the field of view of the individual. The third dimension
	 * will be applied to the intensity value of the chemical. */
	pub green_bias: [f32; 3],

	/** Four-dimensional bias vector for the blue chemical.
	 *
	 * The first two dimensions will be applied to the gradient direction of
	 * the chemical in the field of view of the individual. The third dimension
	 * will be applied to the intensity value of the chemical. */
	pub blue_bias: [f32; 3],

	/** Four-dimensional weight vector for the red chemical.
	 *
	 * The first two dimensions will be applied to the gradient direction of
	 * the chemical in the field of view of the individual. The third dimension
	 * will be applied to the intensity value of the chemical. */
	pub red_weight: [f32; 3],

	/** Four-dimensional weight vector for the red chemical.
	 *
	 * The first two dimensions will be applied to the gradient direction of
	 * the chemical in the field of view of the individual. The third dimension
	 * will be applied to the intensity value of the chemical. */
	pub green_weight: [f32; 3],

	/** Four-dimensional weight vector for the red chemical.
	 *
	 * The first two dimensions will be applied to the gradient direction of
	 * the chemical in the field of view of the individual. The third dimension
	 * will be applied to the intensity value of the chemical. */
	pub blue_weight:  [f32; 3],

	/** Pheromone deposit bias. */
	pub deposit_bias: [f32; 3],

	/** Movement vector bias. */
	pub movement_bias: [f32; 2]
}
impl Individual {
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
		written += write_vec(buf, self.position);
		written += write_vec(buf, self.velocity);

		written += write_vec(buf, self.red_bias);
		written += write_pad(buf, 4);
		written += write_vec(buf, self.green_bias);
		written += write_pad(buf, 4);
		written += write_vec(buf, self.blue_bias);
		written += write_pad(buf, 4);


		written += write_vec(buf, self.red_weight);
		written += write_pad(buf, 4);
		written += write_vec(buf, self.green_weight);
		written += write_pad(buf, 4);
		written += write_vec(buf, self.blue_weight);
		written += write_pad(buf, 4);

		written += write_vec(buf, self.deposit_bias);
		written += write_pad(buf, 4);
		written += write_vec(buf, self.movement_bias);

		written
	}
}

/** Writes the given number of zero bytes into the buffer. */
fn write_pad(buf: &mut Vec<u8>, size: usize) -> usize {
	let new = buf.len().checked_add(size)
		.expect("WHY DO YOU NEED THIS MANY BYTES FOR PADDING?");
	buf.resize(new, 0);

	size
}

/** Writes the given float vector into the buffer. */
fn write_vec<const N: usize>(buf: &mut Vec<u8>, data: [f32; N]) -> usize {
	(0..data.len()).into_iter()
		.map(|i| data[i])
		.map(|p| f32::to_le_bytes(p))
		.map(|bytes| {
			buf.extend_from_slice(&bytes[..]);
			bytes.len()
		})
		.sum()
}
