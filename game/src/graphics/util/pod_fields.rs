//! Converting structs with padding into raw bytes. 

use std::{
    borrow::BorrowMut,
    mem::size_of,
};
use bytemuck::{Pod, bytes_of};

/// A struct with plain-old-data fields, and potentially padding. 
///
/// Whereas `bytemuck::Pod` is an unsafe trait, and cannot be implemented 
/// for types with padding, this is an entirely safe trait which builds off 
/// of `Pod` and can convert structs with padding into fully initialized
/// byte arrays. 
pub trait PodFields {
    fn visit_fields<V: PodFieldVisitor>(&self, visitor: &mut V);
}

pub trait PodFieldVisitor {
    fn visit<F: Pod>(&mut self, field: &F);
}

/// Convert an `impl PodFields` into a fully initialized byte array. 
///
/// `write_to` must be equal in length to `std::mem::size_of::<P>()`.
pub fn copy_bytes_to<P>(structure: &P, write_to: &mut [u8])
where
    P: PodFields,
{
    let bytes = write_to.borrow_mut();
    assert_eq!(bytes.len(), size_of::<P>());

    struct V<'a> {
        start_addr: usize,
        bytes: &'a mut [u8],
    };
    impl<'a> PodFieldVisitor for V<'a> {
        fn visit<F: Pod>(&mut self, field: &F) {
            let field_addr = field as *const F as usize;
            let field_size = size_of::<F>();
            assert!(
                field_size > 0,
                "field ZST not supported"
            );
            assert!(
                field_addr >= self.start_addr,
                "field memory out of bounds"
            );
            assert!(
                field_addr + field_size <= self.start_addr + self.bytes.len(),
                "field memory out of bounds"
            );
            let field_offset = field_addr - self.start_addr;
            for (i, b) in bytes_of(field).iter().copied().enumerate() {
                self.bytes[field_offset + i] = b;
            }
        }
    }

    structure.visit_fields(&mut V {
        start_addr: structure as *const P as usize,
        bytes,
    });
}

#[test]
fn test_pod_fields() {
    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct Foo {
        a: u8,
        b: u16,
    }

    impl PodFields for Foo {
        fn visit_fields<V: PodFieldVisitor>(&self, visitor: &mut V) {
            visitor.visit(&self.a);
            visitor.visit(&self.b);
        }
    }

    let foo = Foo { a: 0xAB, b: 0xCDEF };
    let mut foo_bytes = [0; 4];
    copy_bytes_to(&foo, &mut foo_bytes);
    let correct = [0xAB, 0x00, 0xEF, 0xCD];
    assert_eq!(foo_bytes, correct);
}