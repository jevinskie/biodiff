use std::{ffi::c_int, marker::PhantomData};

use biodiff_wfa2_sys::*;
use serde::{Deserialize, Serialize};

use super::{Align, AlignAlgorithm, CheckStatus, InternalMode};

fn settings(
    algo: &AlignAlgorithm,
    mode: InternalMode,
    text_len: usize,
) -> wavefront_aligner_attr_t {
    let mut attributes = unsafe { wavefront_aligner_attr_default };

    attributes.heuristic.strategy = wf_heuristic_strategy_wf_heuristic_none;
    attributes.alignment_scope = alignment_scope_t_compute_alignment;

    attributes.distance_metric = distance_metric_t_gap_affine;
    attributes.affine_penalties.gap_opening = -algo.gap_open as c_int;
    attributes.affine_penalties.gap_extension = -algo.gap_extend as c_int;
    attributes.affine_penalties.mismatch = -algo.mismatch_score as c_int;
    attributes.affine_penalties.match_ = -algo.match_score as c_int;

    match mode {
        InternalMode::Global => {
            attributes.alignment_form.span = alignment_span_t_alignment_end2end;
            attributes.memory_mode = wavefront_memory_t_wavefront_memory_ultralow;
        }
        InternalMode::Semiglobal => {
            attributes.alignment_form.span = alignment_span_t_alignment_endsfree;
            attributes.alignment_form.pattern_begin_free = 0;
            attributes.alignment_form.pattern_end_free = 0;
            attributes.alignment_form.text_begin_free = text_len as c_int;
            attributes.alignment_form.text_end_free = text_len as c_int;
            // wfa2 currently only supports global alignment for ultra low memory mode
            attributes.memory_mode = wavefront_memory_t_wavefront_memory_high;
        }
    }

    attributes
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Wfa2;

pub struct Aligner<'a>(*mut wavefront_aligner_t, PhantomData<&'a ()>);

impl<'a> Aligner<'a> {
    fn new(settings: &mut wavefront_aligner_attr_t, x: &[u8], y: &[u8]) -> Self {
        let aligner = unsafe { wavefront_aligner_new(settings) };
        if aligner.is_null() {
            panic!("could not create aligner");
        }
        unsafe {
            wavefront_align(
                aligner,
                x.as_ptr() as *const i8,
                x.len() as c_int,
                y.as_ptr() as *const i8,
                y.len() as c_int,
            )
        };
        Self(aligner, PhantomData)
    }
    unsafe fn ops(&self) -> &[u8] {
        let cigar = (*self.0).cigar.as_ref().unwrap();
        let slice = cigar_op_slice(cigar);
        slice
    }
}

impl<'a> Drop for Aligner<'a> {
    fn drop(&mut self) {
        unsafe { wavefront_aligner_delete(self.0) };
    }
}

unsafe fn cigar_op_slice(cigar: &cigar_t) -> &[u8] {
    let begin_ptr = (cigar.operations as *const u8).offset(cigar.begin_offset as isize);
    let len = cigar.end_offset - cigar.begin_offset;
    std::slice::from_raw_parts(begin_ptr, len as usize)
}

const SIZE_LIMIT: u64 = 1 << 30;

impl Align for Wfa2 {
    fn align(
        &self,
        algo: &AlignAlgorithm,
        mode: InternalMode,
        x: &[u8],
        y: &[u8],
    ) -> Vec<bio::alignment::AlignmentOperation> {
        let mut align_attr = settings(algo, mode, y.len());
        let aligner = Aligner::new(&mut align_attr, x, y);
        let mut ret = vec![];
        let slice = unsafe { aligner.ops() };
        for &c in slice {
            match c {
                b'M' => ret.push(bio::alignment::AlignmentOperation::Match),
                b'I' => ret.push(bio::alignment::AlignmentOperation::Del),
                b'D' => ret.push(bio::alignment::AlignmentOperation::Ins),
                b'X' => ret.push(bio::alignment::AlignmentOperation::Subst),
                _ => panic!("unknown cigar operation: {c:x}"),
            }
        }
        ret
    }

    fn check_params(
        &self,
        _: &AlignAlgorithm,
        mode: InternalMode,
        x_size: usize,
        y_size: usize,
    ) -> CheckStatus {
        // for global alignment, we use biwfa, but we use regular wfa which uses quadratic memory
        if matches!(mode, InternalMode::Semiglobal) && x_size as u64 * y_size as u64 > SIZE_LIMIT {
            return CheckStatus::MemoryWarning;
        }
        CheckStatus::Ok
    }
}
