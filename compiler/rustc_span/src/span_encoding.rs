// Spans are encoded using 1-bit tag and 2 different encoding formats (one for each tag value).
// One format is used for keeping span data inline,
// another contains index into an out-of-line span interner.
// The encoding format for inline spans were obtained by optimizing over crates in rustc/libstd.
// See

use crate::def_id::{DefIndex, LocalDefId};
use crate::hygiene::SyntaxContext;
use crate::SPAN_TRACK;
use crate::{BytePos, SpanData};

use rustc_data_structures::fx::FxIndexSet;

/// A compressed span.
///
/// Whereas [`SpanData`] is 16 bytes, which is a bit too big to stick everywhere, `Span`
/// is a form that only takes up 8 bytes, with less space for the length, parent and
/// context. The vast majority (99.9%+) of `SpanData` instances will fit within
/// those 8 bytes; any `SpanData` whose fields don't fit into a `Span` are
/// stored in a separate interner table, and the `Span` will index into that
/// table. Interning is rare enough that the cost is low, but common enough
/// that the code is exercised regularly.
///
/// An earlier version of this code used only 4 bytes for `Span`, but that was
/// slower because only 80--90% of spans could be stored inline (even less in
/// very large crates) and so the interner was used a lot more.
///
/// Inline (compressed) format with no parent:
/// - `span.base_or_index == span_data.lo`
/// - `span.len_or_tag == len == span_data.hi - span_data.lo` (must be `<= MAX_LEN`)
/// - `span.ctxt_or_tag == span_data.ctxt` (must be `<= MAX_CTXT`)
///
/// Interned format with inline `SyntaxContext`:
/// - `span.base_or_index == index` (indexes into the interner table)
/// - `span.len_or_tag == LEN_TAG` (high bit set, all other bits are zero)
/// - `span.ctxt_or_tag == span_data.ctxt` (must be `<= MAX_CTXT`)
///
/// Inline (compressed) format with root context:
/// - `span.base_or_index == span_data.lo`
/// - `span.len_or_tag == len == span_data.hi - span_data.lo` (must be `<= MAX_LEN`)
/// - `span.len_or_tag` has top bit (`PARENT_MASK`) set
/// - `span.ctxt == span_data.parent` (must be `<= MAX_CTXT`)
///
/// Interned format:
/// - `span.base_or_index == index` (indexes into the interner table)
/// - `span.len_or_tag == LEN_TAG` (high bit set, all other bits are zero)
/// - `span.ctxt_or_tag == CTXT_TAG`
///
/// The inline form uses 0 for the tag value (rather than 1) so that we don't
/// need to mask out the tag bit when getting the length, and so that the
/// dummy span can be all zeroes.
///
/// Notes about the choice of field sizes:
/// - `base` is 32 bits in both `Span` and `SpanData`, which means that `base`
///   values never cause interning. The number of bits needed for `base`
///   depends on the crate size. 32 bits allows up to 4 GiB of code in a crate.
/// - `len` is 15 bits in `Span` (a u16, minus 1 bit for the tag) and 32 bits
///   in `SpanData`, which means that large `len` values will cause interning.
///   The number of bits needed for `len` does not depend on the crate size.
///   The most common numbers of bits for `len` are from 0 to 7, with a peak usually
///   at 3 or 4, and then it drops off quickly from 8 onwards. 15 bits is enough
///   for 99.99%+ of cases, but larger values (sometimes 20+ bits) might occur
///   dozens of times in a typical crate.
/// - `ctxt_or_tag` is 16 bits in `Span` and 32 bits in `SpanData`, which means that
///   large `ctxt` values will cause interning. The number of bits needed for
///   `ctxt` values depend partly on the crate size and partly on the form of
///   the code. No crates in `rustc-perf` need more than 15 bits for `ctxt_or_tag`,
///   but larger crates might need more than 16 bits.
///
/// In order to reliably use parented spans in incremental compilation,
/// the dependency to the parent definition's span. This is performed
/// using the callback `SPAN_TRACK` to access the query engine.
///
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[rustc_pass_by_value]
pub struct Span {
    base_or_index: u32,
    len_or_tag: u16,
    ctxt_or_tag: u16,
}

// #[derive(Clone, Copy, Eq, PartialEq, Hash)]
// #[rustc_pass_by_value]
// pub struct SpanChain {
//     base_or_index: u32,
//     len_or_tag: u16,
//     ctxt_or_tag: u16,
// }

/// The dummy span has zero position, length, and context, and no parent.
pub const DUMMY_SP: Span =
    Span { lo_or_index: 0, len_with_tag_or_marker: 0, ctxt_or_parent_or_marker: 0 };
<<<<<<< HEAD
// #[derive(Clone, Copy, Eq, PartialEq, Hash)]
// #[rustc_pass_by_value]
// pub struct SpanChain {
//     base_or_index: u32,
//     len_or_tag: u16,
//     ctxt_or_tag: u16,
// }

/// Dummy span, both position and length are zero, syntax context is zero as well.

// pub const DUMMY_SP_CH: SpanChain = SpanChain {base_or_index: 0, len_or_tag: 0, ctxt_or_tag: 0};

/// Dummy span, both position and length are zero, syntax context is zero as well.

// pub const DUMMY_SP_CH: SpanChain = SpanChain {base_or_index: 0, len_or_tag: 0, ctxt_or_tag: 0};

/// Dummy span, both position and length are zero, syntax context is zero as well.

/// Dummy span, both position and length are zero, syntax context is zero as well.

=======

/// Dummy span, both position and length are zero, syntax context is zero as well.

>>>>>>> 218b760d0d8 (Update changes)
impl Span {
    #[inline]
    pub fn new(
        mut lo: BytePos,
        mut hi: BytePos,
        ctxt: SyntaxContext,
        parent: Option<LocalDefId>,
    ) -> Self {
        if lo > hi {
            std::mem::swap(&mut lo, &mut hi);
        }

        let (base, len, ctxt2) = (lo.0, hi.0 - lo.0, ctxt.as_u32());

        if len <= MAX_LEN && ctxt2 <= MAX_CTXT {
            let len_or_tag = len as u16;
            debug_assert_eq!(len_or_tag & PARENT_MASK, 0);

            if let Some(parent) = parent {
                // Inline format with parent.
                let len_or_tag = len_or_tag | PARENT_MASK;
                let parent2 = parent.local_def_index.as_u32();
                if ctxt2 == SyntaxContext::root().as_u32()
                    && parent2 <= MAX_CTXT
                    && len_or_tag < LEN_TAG
                {
                    debug_assert_ne!(len_or_tag, LEN_TAG);
                    return Span { base_or_index: base, len_or_tag, ctxt_or_tag: parent2 as u16 };
                }
            } else {
                // Inline format with ctxt.
                debug_assert_ne!(len_or_tag, LEN_TAG);
                return Span {
                    lo_or_index: lo2,
                    len_with_tag_or_marker: len as u16,
                    ctxt_or_parent_or_marker: ctxt2 as u16,
                };
            }
            if ctxt2 == SyntaxContext::root().as_u32()
                && let Some(parent) = parent
                && let parent2 = parent.local_def_index.as_u32()
                && parent2 <= MAX_CTXT
            {
                // Inline-parent format.
                return Span {
                    lo_or_index: lo2,
                    len_with_tag_or_marker: PARENT_TAG | len as u16,
                    ctxt_or_parent_or_marker: parent2 as u16,
                };
            }
        }

        // Interned format.
        let index =
            with_span_interner(|interner| interner.intern(&SpanData { lo, hi, ctxt, parent }));
        let ctxt_or_tag = if ctxt2 <= MAX_CTXT { ctxt2 } else { CTXT_TAG } as u16;
        Span { base_or_index: index, len_or_tag: LEN_TAG, ctxt_or_tag }
    }

    #[inline]
    pub fn data(self) -> SpanData {
        let data = self.data_untracked();
        if let Some(parent) = data.parent {
            (*SPAN_TRACK)(parent);
        }
        data
    }

    /// Internal function to translate between an encoded span and the expanded representation.
    /// This function must not be used outside the incremental engine.
    #[inline]
    pub fn data_untracked(self) -> SpanData {
        if self.len_or_tag != LEN_TAG {
            // Inline format.
            if self.len_or_tag & PARENT_MASK == 0 {
                debug_assert!(self.len_or_tag as u32 <= MAX_LEN);
                SpanData {
                    lo: BytePos(self.base_or_index),
                    hi: BytePos(self.base_or_index + self.len_or_tag as u32),
                    ctxt: SyntaxContext::from_u32(self.ctxt_or_tag as u32),
                    parent: None,
                }
            } else {
                let len = self.len_or_tag & !PARENT_MASK;
                debug_assert!(len as u32 <= MAX_LEN);
                let parent =
                    LocalDefId { local_def_index: DefIndex::from_u32(self.ctxt_or_tag as u32) };
                SpanData {
                    lo: BytePos(self.base_or_index),
                    hi: BytePos(self.base_or_index + len as u32),
                    ctxt: SyntaxContext::root(),
                    parent: Some(parent),
                }
            }
        } else {
            // Interned format.
            let index = self.base_or_index;
            with_span_interner(|interner| interner.spans[index as usize])
        }
    }

    /// This function is used as a fast path when decoding the full `SpanData` is not necessary.
    #[inline]
    pub fn ctxt(self) -> SyntaxContext {
        let ctxt_or_tag = self.ctxt_or_tag as u32;
        // Check for interned format.
        if self.len_or_tag == LEN_TAG {
            if ctxt_or_tag == CTXT_TAG {
                // Fully interned format.
                let index = self.base_or_index;
                with_span_interner(|interner| interner.spans[index as usize].ctxt)
            } else {
                // Interned format with inline ctxt.
                SyntaxContext::from_u32(ctxt_or_tag)
            }
        } else if self.len_or_tag & PARENT_MASK == 0 {
            // Inline format with inline ctxt.
            SyntaxContext::from_u32(ctxt_or_tag)
        } else {
            // Inline format with inline parent.
            // We know that the SyntaxContext is root.
            SyntaxContext::root()
        }
    }
}
/*
impl SpanChain {
    #[inline]
    pub fn new(spans: Vec<Span>) -> Self {
        if spans.len() == 1 {
            let (base, len, ctxt) = (
                spans[0].lo_or_index,
                spans[0].len_with_tag_or_marker,
                spans[0].ctxt_or_parent_or_marker,
            );
            return SpanChain { base_or_index: base, len_or_tag: len, ctxt_or_tag: ctxt };
        } else {
            let index =
                with_span_chain_interner(|span_chain_interner| span_chain_interner.intern(spans));
            return SpanChain {
                base_or_index: index,
                len_or_tag: MAX_LEN as u16,
                ctxt_or_tag: CTXT_TAG as u16,
            };
        }
    }

    #[inline]
    pub fn new1(
        mut lo: BytePos,
        mut hi: BytePos,
        ctxt: SyntaxContext,
        parent: Option<LocalDefId>,
    ) -> Self {
        let tmp = Span::new(lo, hi, ctxt, parent);
        SpanChain::new(vec![tmp])
    }

    #[inline]
    pub fn data(self) -> SpanData {
        let data = self.data_untracked();
        if let Some(parent) = data.parent {
            (*SPAN_TRACK)(parent);
        }
        data
    }

    #[inline]
    pub fn data_untracked(self) -> SpanData {
        if self.len_or_tag != MAX_LEN as u16 {
            let (base, len, ctx) = (self.base_or_index, self.len_or_tag, self.ctxt());
            let tmp =
                Span { base_or_index: base, len_or_tag: len, ctxt_or_tag: ctx.as_u32() as u16 };
            tmp.data_untracked()
        } else {
            let index = self.base_or_index;
            let tmp = with_span_chain_interner(|chain_interner| {
                chain_interner.spans_chain[index as usize]
            });
            tmp[0].data_untracked()
        }
    }

    #[inline]
    pub fn ctxt(self) -> SyntaxContext {
        if self.len_or_tag != MAX_LEN as u16 {
            let (base, len, ctx) = (self.base_or_index, self.len_or_tag, self.ctxt());
            let tmp =
                Span { base_or_index: base, len_or_tag: len, ctxt_or_tag: ctx.as_u32() as u16 };
            tmp.ctxt()
        } else {
            let index = self.base_or_index;
            let tmp = with_span_chain_interner(|chain_interner| {
                chain_interner.spans_chain[index as usize]
            });
            tmp[0].ctxt()
        }
    }

    #[inline]
    pub fn to_span(self) -> Span {
        if self.len_or_tag != MAX_LEN as u16 {
            let (base, len, ctx) = (self.base_or_index, self.len_or_tag, self.ctxt());
            Span {
                lo_or_index: base,
                len_with_tag_or_marker: len,
                ctxt_or_parent_or_marker: ctx.as_u32() as u16,
            }
        } else {
            let index = self.base_or_index;
            let tmp = with_span_chain_interner(|chain_interner| {
                chain_interner.spans_chain[index as usize]
            });
            tmp[0]
        }
    }
}
*/

#[derive(Default)]
pub struct SpanInterner {
    spans: FxIndexSet<SpanData>,
}
/*
#[derive(Default)]
pub struct SpanChainInterner {
    spans_chain: FxIndexSet<Vec<Span>>,
}
*/
<<<<<<< HEAD

=======
>>>>>>> 218b760d0d8 (Update changes)

impl SpanInterner {
    fn intern(&mut self, span_data: &SpanData) -> u32 {
        let (index, _) = self.spans.insert_full(*span_data);
        index as u32
    }
}

/*
impl SpanChainInterner {
    fn intern(&mut self, span_chain_data: Vec<Span>) -> u32 {
        let (index, _) = self.spans_chain.insert_full(span_chain_data);
        index as u32
    }
}
*/

// If an interner exists, return it. Otherwise, prepare a fresh one.
#[inline]
fn with_span_interner<T, F: FnOnce(&mut SpanInterner) -> T>(f: F) -> T {
    crate::with_session_globals(|session_globals| f(&mut session_globals.span_interner.lock()))
}

/*
#[inline]
fn with_span_chain_interner<T, F: FnOnce(&mut SpanChainInterner) -> T>(f: F) -> T {
    crate::with_session_globals(|session_globals| {
        f(&mut session_globals.span_chain_interner.lock())
    })
}
*/
