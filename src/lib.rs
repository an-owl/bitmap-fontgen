#[cfg(not(feature = "std"))]
use core as std;

use std::fmt::{Display, Formatter};
use std::hash::Hasher;
use phf_shared::{FmtConst, PhfBorrow, PhfHash};

#[cfg(feature = "codegen")]
pub mod codegen;

/// Represents the font's weight.
/// Usually this includes "Normal" and "Bold" weights.
/// This crate does not specifically define any weights, users of this library are required to know
/// the weights they are using.
// Todo users of this library may want to ensure that all weights are spelled the same, a test is provided for this.
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Copy, Clone)]
pub struct FontWeight {
    inner: &'static str,
}

impl Display for FontWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.inner)
    }
}

impl From<&'static str> for FontWeight {
    fn from(value: &'static str) -> Self {
        Self {inner: value}
    }
}

impl PhfHash for FontWeight {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        for i in self.inner.chars() {
            state.write_u32(i as u32)
        }
    }
}

impl PhfBorrow<FontWeight> for FontWeight {
    fn borrow(&self) -> &FontWeight {
        self
    }
}

/// Represents the size of a character glyph.
///
/// Ord is derived on this struct to enable sorting it, the result noes not necessarily mean anything.
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "codegen", derive(Hash))]
pub struct FontSize {
    width: u16,
    height: u16,
}
pub struct Font {
    pub font: &'static phf::Map<FontWeight,phf::Map<FontSize,phf::Map<char,&'static [u8]>>> // what a mess
}

impl PhfBorrow<FontSize> for FontSize {
    fn borrow(&self) -> &FontSize {
        self
    }
}

impl PhfHash for FontSize {
    fn phf_hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(self.width);
        state.write_u16(self.height);
    }
}

#[cfg(feature = "codegen")]
impl FmtConst for FontSize {
    fn fmt_const(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"::{}::{:?}",module_path!(),self)
    }
}


impl Font {
    /// Returns all font weights available
    pub fn weights(&self) -> Vec<FontWeight> {
        self.font.keys().map(|e| *e).collect()
    }

    /// Returns the font sizes and whether they are available for all weights.
    pub fn sizes(&self) -> Vec<(FontSize,bool)> {
        let mut cmp: Vec<Vec<FontSize>> = Vec::new();
        for (_,i) in self.font{
            cmp.push(i.keys().map(|e| *e).collect())
        };

        for i in &mut cmp {
            i.sort();
        }
        let needle = cmp.pop().expect("Why is this empty (•ิ_•ิ)?");
        let haystack = cmp;
        let mut ret = Vec::new();

        'needle: for i in needle {
            'haystack: for j in &haystack {
                match j.binary_search(&i) {
                    Ok(_) => continue 'haystack,
                    Err(_) => {
                        ret.push((i, false));
                        continue 'needle; // skips pushing (i,true)
                    }
                }
            }
            ret.push((i,true))
        }

        ret.append(&mut haystack.iter().flatten().map(|e| (*e,false)).collect::<Vec<(FontSize,bool)>>()); // flattens 2d arr into 1d. maps each so each element
        ret.sort_by(|(e,_), (u,_)| e.cmp(u) ); // sorts ignoring the bool
        ret.dedup();
        ret
    }

    fn get(&self, weight: FontWeight, size: FontSize, ch: char ) -> Option<BitMap> {
        Some(
            BitMap {
                size,
                map: self.font.get(&weight)?.get(&size)?.get(&ch).unwrap()
            }
        )
    }
}

struct BitMap {
    size: FontSize,
    map: &'static [u8]
}

impl BitMap {

    /// Returns the raw bitmap
    fn raw(&self) -> &'static [u8] {
        self.map
    }

    /// Returns the size of the font face.
    fn size(&self) -> FontSize {
        self.size
    }

    /// Converts the bitmap using calling `f` on each pixel to perform the conversion and writing the result into the buffer.
    fn convert<F, T>(&self, f: F, buff: &mut [T])
        where F: Fn(bool) -> T
    {
        for i in 0..(self.size.width * self.size.height) as usize {
            let byte = i / u8::BITS as usize;
            let bit = i % u8::BITS as usize;
            buff[i] = f(self.map[byte] & 1<<bit != 0);
        }
    }

    /// Converts the bitmap calling `f` to perform the conversion similarly to [Self::convert].
    /// `f` is intentionally the same used by [Self::convert].
    /// This version takes a 2 dimensional slice allowing the bitmap do be directly drawn into a
    /// framebuffer where the first dimension is the scan line and the second is the column `buff\[scan\]\[px\].
    ///
    /// The caller should ensure that each slice points to the same column otherwise the glyph will be drawn incorrectly
    ///
    /// # Panics
    ///
    /// This fn will panic if `buff` does not provide enough scan lines or pixels in a scan line
    fn convert_fb<F,T>(&self, f: F, buff: &mut [&mut[T]])
        where F: Fn(bool) -> T
    {
        for scan in 0..self.size.height as usize {
            for p in 0..self.size.width as usize {
                let byte = p / u8::BITS as usize;
                let bit = p % u8::BITS as usize;
                buff[scan][p] = f(self.map[byte] & 1 << bit != 0)
            }
        }
    }
}