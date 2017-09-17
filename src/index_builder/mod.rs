pub mod backend;


#[allow(missing_docs)]
pub trait Backend {
    fn create_full_bitmap(&self, s: &[u8], offset: usize) -> Bitmap;
    fn create_partial_bitmap(&self, s: &[u8], offset: usize) -> Bitmap;
}


#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub struct Bitmap {
    backslash: u64,
    quote: u64,
    colon: u64,
    left_brace: u64,
    right_brace: u64,
}

#[allow(missing_docs)]
pub fn build_structural_character_bitmaps<B: Backend + Default>(s: &[u8]) -> Vec<Bitmap> {
    let backend = B::default();

    let mut result = Vec::with_capacity((s.len() + 63) / 64);

    for i in 0..(s.len() / 64) {
        result.push(backend.create_full_bitmap(s, i * 64));
    }

    if s.len() % 64 != 0 {
        result.push(backend.create_partial_bitmap(s, (s.len() / 64) * 64));
    }

    result
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::backend::Sse2Backend;

    #[test]
    fn test_structural_character_bitmaps() {
        struct TestCase {
            input: &'static [u8],
            expected: Vec<Bitmap>,
        }
        let cases = vec![
            TestCase {
                input: b"{}",
                expected: vec![
                    Bitmap {
                        backslash: 0,
                        quote: 0,
                        colon: 0,
                        left_brace: 0b0000_0001,
                        right_brace: 0b0000_0010,
                    },
                ],
            },
            TestCase {
                input: r#"{"x\"y\\":10}"#.as_bytes(),
                expected: vec![
                    Bitmap {
                        backslash: 0b1100_1000,
                        quote: 0b0001_0001_0010,
                        colon: 0b0010_0000_0000,
                        left_brace: 0b0000_0001,
                        right_brace: 0b0001_0000_0000_0000,
                    },
                ],
            },
        ];

        for case in cases {
            let actual = build_structural_character_bitmaps::<Sse2Backend>(case.input);
            assert_eq!(&actual[..], &case.expected[..]);
        }
    }
}
