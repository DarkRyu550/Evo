use std::time::{Instant, Duration};
use std::convert::TryInto;
use std::sync::atomic::AtomicU64;

pub struct Carriage<T> {
	storage: [T; 3],
	baseline: Instant,

	producer: AtomicU64,
	consumer: AtomicU64,
	limbo: AtomicU64,
}
impl<T> Carriage<T> {
	pub fn
}

/** This structure is a pointer adapter, responsible for packing internal
 * pointers within the carriage into `u64` values, so that they may be used for
 * atomic operations.
 * # Pointer
 * The information that constitutes a pointer, in this context, is an index into
 * the storage array of the carriage, along with a nanosecond-precise timestamp.
 * # Ordering of the resulting `u64`
 * When comparing two packed values, the layout is such that the comparison will
 * behave the same as if you were to first rank the timestamps, followed by the
 * indices. With older values ranking greater, regardless of their indices, and
 * values with the same nanosecond age but greater indices ranking greater.
 */
struct U64PointerPacker {
	index: u8,
	timestamp: Duration,
}
impl U64PointerPacker {
	/** Creates a new pointer from an index and a timestamp.
	 *
	 * As this type is mainly intended for packing data for atomic operations,
	 * it is important to note it has limits on its accepted values.
	 * # Limits
	 * The index must not be bigger than `0xff` and the timestamp must not be
	 * longer than `0x00ff_ffff_ffff_ffff` nanoseconds.
	 * # Panic
	 * This function panics if either of the limits are not respected.
	 */
	pub fn new(index: usize, timestamp: Duration) -> Self {

	}

	/** The carriage storage index this points to. */
	pub fn index(&self) -> usize {
		self.index.try_into()
			.expect("the index should be a valid usize by now")
	}

	/** The timestamp associated with this pointer. Usually the time since the
	 * pointer was inserted into to the carriage by means of its modification
	 * functions. */
	pub fn timestamp(&self) -> Duration {
		self.timestamp
	}
}
impl From<u64> for U64PointerPacker {
	fn from(val: u64) -> Self {
		let index = (val & 0xff) as u8;

		let timestamp = (val >> 8 & 0x00ff_ffff_ffff_ffff);
		let timestamp = Duration::from_nanos(timestamp);

		Self { index, timestamp }
	}
}
impl Into<u64> for U64PointerPacker {
	fn into(self) -> u64 {
		let mut res = 0_u64;

		let nanos = self.timestamp.as_nanos();
		if nanos > 0x00ff_ffff_ffff_ffff {
			/* It is bad etiquette having into() panic. But these checks should
			 * have already been performed earlier. It is a fatal bug if any
			 * invalid timestamp actually manages to get to this point. */
			panic!("the timestamp should already fit in 56 bits by this point");
		}

		res  |= (nanos & 0x00ff_ffff_ffff_ffff) as u64;
		res <<= 8;
		res  |= u64::from(self.index);

		res
	}
}
