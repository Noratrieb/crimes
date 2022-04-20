// we are law abiding citizens using only the best features of the strict provenance world
use sptr::Strict;
// used to store provenance
use std::mem::MaybeUninit;

// writing a cfged version for 32 bit is trivial, we don't care
#[cfg(not(target_pointer_width = "64"))]
compile_error!("not supported");

// a pointer sized buffer used to store a single pointer
type Buf = MaybeUninit<[u8; 8]>;
// a double pointer sized buffer used to store two pointers and rip out the center
type DBuf = MaybeUninit<[u8; 16]>;

/// just a pointer, doesn't matter which one
type Ptr = *const u8;

#[repr(C)]
#[repr(align(8))]
struct Align8<T>(T);

/// combines two provenances into a single pointer sized value
unsafe fn combine(prov_a: Ptr, prov_b: Ptr) -> Buf {
    // a buffer where we write in both pointers and then read out of from center, 1/2 of each provenance
    let mut double_buf = Align8(DBuf::zeroed());

    let ptr = double_buf.0.as_mut_ptr();

    // write the a pointer to the first slot
    ptr.cast::<Ptr>().write(prov_a);
    // write the b pointer to the second slot
    ptr.cast::<Buf>().add(1).cast::<Ptr>().write(prov_b);

    // and read out the center
    let center = ptr.cast::<u8>().add(4).cast::<Buf>().read();
    center
}

/// extracts the two provenances from [`combine`]
unsafe fn extract(buf: Buf) -> (Ptr, Ptr) {
    let mut double_buf = Align8(DBuf::zeroed());

    // write the the pointer sized value into the center of the double buffer
    // splitting the provenances between the first and second slow
    double_buf
        .0
        .as_mut_ptr()
        .cast::<u8>()
        .add(4)
        .cast::<Buf>()
        .write(buf);

    // a copy of the first half of the dbuf, where the second half of it contains a provenance
    let mut a_buf: Buf = double_buf.0.as_ptr().cast::<Buf>().read();
    // a copy of the second half of the dbuf, where the first half of it contains b provenance
    let mut b_buf: Buf = double_buf.0.as_ptr().cast::<Buf>().add(1).read();

    // the pointer to the dbuf
    let ptr = double_buf.0.as_ptr();

    // copy the 4 a provenance bytes from the dbuf into the empty space in the a_buf
    // this way a_buf now contains 8 a provenance bytes
    std::ptr::copy_nonoverlapping(
        ptr.cast::<MaybeUninit<u8>>().add(4),
        a_buf.as_mut_ptr().cast::<MaybeUninit<u8>>(),
        4,
    );

    // repeat the same thing for the b provenance
    std::ptr::copy_nonoverlapping(
        ptr.cast::<MaybeUninit<u8>>().add(8),
        b_buf.as_mut_ptr().cast::<MaybeUninit<u8>>().add(4),
        4,
    );

    // both buffers are now filled with fancy provenance bytes, read the pointers out and return them
    let a = a_buf.as_ptr().cast::<Ptr>().read();
    let b = b_buf.as_ptr().cast::<Ptr>().read();

    (a, b)
}

fn main() {
    unsafe {
        // two innocent looking integers
        let a = 5u8;
        let b = 3u8;

        // two innocent looking pointers
        let a_ptr = &a as Ptr;
        let b_ptr = &b as Ptr;

        // extract the addresses for later use. in the real xorlist, these would be xored
        let a_addr = Strict::addr(a_ptr);
        let b_addr = Strict::addr(b_ptr);

        // if we were implementing an actual xorlist, we would be setting the addresses of
        // the pointers to our xored address so that the combined buffer stores the full address
        // and we could get it out through more complex magic. this is besides the point of this
        // demonstration, it is only concerned with combining provenances

        // combine the provenances
        let cursed = combine(a_ptr, b_ptr);

        // do crimes here

        // and get them out again
        let (a_prov, b_prov) = extract(cursed);

        // make them pointers again!
        let new_a = Strict::with_addr(a_prov, a_addr);
        let new_b = Strict::with_addr(b_prov, b_addr);

        // it works now, right? :ferrisClueless:
        assert_eq!(*new_a, 5);
        assert_eq!(*new_b, 3);
    }
}
