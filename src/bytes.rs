// this was actually quite fun to write!
// generic byte writing (based on networking
// principles) thru channels!
// easy to swap out the type later on, if needed
// (it probably won't be swapped out)

/// `N`: MTU (size of buffer)
pub struct FrameN<const N: usize> {
    // in order to not send an MTU-sized frame each time
    // "b-but muh extra byte!!!"
    // shut the fuck up
    data_len: u8,
    data: [u8; N], // MTU
}

pub trait IntoBytes<const N: usize> {
    /// `N`: MTU
    fn from_buf(buf: [u8; N], len: usize) -> Self;
}

pub trait FromBytes {
    fn into_slice(&self) -> &[u8];
}

impl<const N: usize, const M: usize> IntoBytes<M> for FrameN<N> {
    fn from_buf(buf: [u8; M], len: usize) -> Self {
        Self {
            data_len: (len as u8),
            data: buf[0..N].try_into().unwrap(),
        }
    }
}

impl<const N: usize> FromBytes for FrameN<N> {
    fn into_slice(&self) -> &[u8] {
        &self.data[0..(self.data_len as usize)]
    }
}
