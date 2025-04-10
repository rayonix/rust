#![allow(unused)]
#![feature(rustc_private)]

extern crate rustc_codegen_ssa;
extern crate rustc_middle;
extern crate rustc_target;

use rustc_codegen_ssa::back::write::CodegenResult;
use rustc_codegen_ssa::traits::*;
use rustc_middle::ty::TyCtxt;
use rustc_target::spec::Target;

use crate::context::CodegenCx;

use melior::Context;

pub struct Builder;
impl BuilderMethods for Builder {
    type CodegenCx = CodegenCx; // MLIR context for code generation operations

    // Add stubs for all required methods
    fn build(&mut self) {
        unimplemented!()
    }
    fn cx(&self) -> &Self::CodegenCx {
        unimplemented!()
    }
    fn llbb(&self) -> &<Self::CodegenCx as BackendTypes>::BasicBlock {
        unimplemented!()
    }
    fn set_span(&mut self, _span: rustc_span::Span) {
        unimplemented!()
    }
    fn append_block(
        &mut self,
        _function: &<Self::CodegenCx as BackendTypes>::Function,
        _name: &str,
    ) -> <Self::CodegenCx as BackendTypes>::BasicBlock {
        unimplemented!()
    }
    fn append_sibling_block(
        &mut self,
        _bb: &<Self::CodegenCx as BackendTypes>::BasicBlock,
    ) -> <Self::CodegenCx as BackendTypes>::BasicBlock {
        unimplemented!()
    }
    fn switch_to_block(&mut self, _bb: <Self::CodegenCx as BackendTypes>::BasicBlock) {
        unimplemented!()
    }
    fn ret_void(&mut self) {
        unimplemented!()
    }
    fn ret(&mut self, _value: <Self::CodegenCx as BackendTypes>::Value) {
        unimplemented!()
    }
    fn br(&mut self, _dest: <Self::CodegenCx as BackendTypes>::BasicBlock) {
        unimplemented!()
    }
    fn cond_br(
        &mut self,
        _cond: <Self::CodegenCx as BackendTypes>::Value,
        _then_bb: <Self::CodegenCx as BackendTypes>::BasicBlock,
        _else_bb: <Self::CodegenCx as BackendTypes>::BasicBlock,
    ) {
        unimplemented!()
    }
    fn switch(
        &mut self,
        _v: <Self::CodegenCx as BackendTypes>::Value,
        _else_bb: <Self::CodegenCx as BackendTypes>::BasicBlock,
        _cases: Vec<(
            <Self::CodegenCx as BackendTypes>::Value,
            <Self::CodegenCx as BackendTypes>::BasicBlock,
        )>,
    ) {
        unimplemented!()
    }
    fn invoke(
        &mut self,
        _llfn: <Self::CodegenCx as BackendTypes>::Value,
        _args: &[<Self::CodegenCx as BackendTypes>::Value],
        _then: <Self::CodegenCx as BackendTypes>::BasicBlock,
        _catch: <Self::CodegenCx as BackendTypes>::BasicBlock,
        _funclet: Option<&<Self::CodegenCx as BackendTypes>::Funclet>,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn unreachable(&mut self) {
        unimplemented!()
    }
    fn add(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fadd(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fadd_fast(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fadd_algebraic(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn sub(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fsub(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fsub_fast(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fsub_algebraic(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn mul(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fmul(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fmul_fast(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fmul_algebraic(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn udiv(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn exactudiv(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn sdiv(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn exactsdiv(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fdiv(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fdiv_fast(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fdiv_algebraic(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn urem(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn srem(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn frem(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn frem_fast(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn frem_algebraic(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn shl(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn lshr(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn ashr(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn and(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn or(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn xor(
        &mut self,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn neg(
        &mut self,
        _value: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fneg(
        &mut self,
        _value: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn not(
        &mut self,
        _value: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn checked_binop(
        &mut self,
        _op: rustc_middle::mir::BinOp,
        _ty: rustc_middle::ty::Ty<'_>,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn from_immediate(
        &mut self,
        _val: rustc_codegen_ssa::common::Immediate,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn to_immediate_scalar(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _scalar_ty: &rustc_target::abi::Scalar,
    ) -> Result<rustc_codegen_ssa::common::Immediate, ()> {
        unimplemented!()
    }
    fn alloca(
        &mut self,
        _ty: <Self::CodegenCx as BackendTypes>::Type,
        _align: rustc_target::abi::Align,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn dynamic_alloca(
        &mut self,
        _ty: <Self::CodegenCx as BackendTypes>::Type,
        _size: <Self::CodegenCx as BackendTypes>::Value,
        _align: rustc_target::abi::Align,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn load(
        &mut self,
        _ty: <Self::CodegenCx as BackendTypes>::Type,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _align: rustc_target::abi::Align,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn volatile_load(
        &mut self,
        _ty: <Self::CodegenCx as BackendTypes>::Type,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _align: rustc_target::abi::Align,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn atomic_load(
        &mut self,
        _ty: <Self::CodegenCx as BackendTypes>::Type,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _order: rustc_codegen_ssa::common::AtomicOrdering,
        _align: rustc_target::abi::Align,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn load_operand(
        &mut self,
        _place: rustc_codegen_ssa::mir::place::PlaceRef<
            '_,
            <Self::CodegenCx as BackendTypes>::Value,
        >,
    ) -> rustc_codegen_ssa::mir::operand::OperandRef<'_, <Self::CodegenCx as BackendTypes>::Value>
    {
        unimplemented!()
    }
    fn write_operand_repeatedly<I, F>(&mut self, _func: F, _iter: I)
    where
        F: FnMut(&mut Self, I::Item),
        I: Iterator,
    {
        unimplemented!()
    }
    fn range_metadata(
        &mut self,
        _load: <Self::CodegenCx as BackendTypes>::Value,
        _range: std::ops::Range<u128>,
    ) {
        unimplemented!()
    }
    fn nonnull_metadata(&mut self, _load: <Self::CodegenCx as BackendTypes>::Value) {
        unimplemented!()
    }
    fn store(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _align: rustc_target::abi::Align,
    ) {
        unimplemented!()
    }
    fn store_with_flags(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _align: rustc_target::abi::Align,
        _flags: rustc_codegen_ssa::MemFlags,
    ) {
        unimplemented!()
    }
    fn atomic_store(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _order: rustc_codegen_ssa::common::AtomicOrdering,
        _align: rustc_target::abi::Align,
    ) {
        unimplemented!()
    }
    fn gep(
        &mut self,
        _ty: <Self::CodegenCx as BackendTypes>::Type,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _indices: &[<Self::CodegenCx as BackendTypes>::Value],
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn inbounds_gep(
        &mut self,
        _ty: <Self::CodegenCx as BackendTypes>::Type,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _indices: &[<Self::CodegenCx as BackendTypes>::Value],
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn trunc(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn sext(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fptoui_sat(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fptosi_sat(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fptoui(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fptosi(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn uitofp(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn sitofp(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fptrunc(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fpext(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn ptrtoint(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn inttoptr(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn bitcast(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn intcast(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
        _is_signed: bool,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn pointercast(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn icmp(
        &mut self,
        _op: rustc_codegen_ssa::common::IntPredicate,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn fcmp(
        &mut self,
        _op: rustc_codegen_ssa::common::RealPredicate,
        _lhs: <Self::CodegenCx as BackendTypes>::Value,
        _rhs: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn memcpy(
        &mut self,
        _dst: <Self::CodegenCx as BackendTypes>::Value,
        _dst_align: rustc_target::abi::Align,
        _src: <Self::CodegenCx as BackendTypes>::Value,
        _src_align: rustc_target::abi::Align,
        _size: <Self::CodegenCx as BackendTypes>::Value,
        _flags: rustc_codegen_ssa::MemFlags,
    ) {
        unimplemented!()
    }
    fn memmove(
        &mut self,
        _dst: <Self::CodegenCx as BackendTypes>::Value,
        _dst_align: rustc_target::abi::Align,
        _src: <Self::CodegenCx as BackendTypes>::Value,
        _src_align: rustc_target::abi::Align,
        _size: <Self::CodegenCx as BackendTypes>::Value,
        _flags: rustc_codegen_ssa::MemFlags,
    ) {
        unimplemented!()
    }
    fn memset(
        &mut self,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _fill_byte: <Self::CodegenCx as BackendTypes>::Value,
        _size: <Self::CodegenCx as BackendTypes>::Value,
        _align: rustc_target::abi::Align,
        _flags: rustc_codegen_ssa::MemFlags,
    ) {
        unimplemented!()
    }
    fn select(
        &mut self,
        _cond: <Self::CodegenCx as BackendTypes>::Value,
        _then_val: <Self::CodegenCx as BackendTypes>::Value,
        _else_val: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn va_arg(
        &mut self,
        _list: <Self::CodegenCx as BackendTypes>::Value,
        _ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn extract_element(
        &mut self,
        _vec: <Self::CodegenCx as BackendTypes>::Value,
        _idx: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn vector_splat(
        &mut self,
        _num_elts: usize,
        _elt: <Self::CodegenCx as BackendTypes>::Value,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn extract_value(
        &mut self,
        _aggregate: <Self::CodegenCx as BackendTypes>::Value,
        _idx: u64,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn insert_value(
        &mut self,
        _aggregate: <Self::CodegenCx as BackendTypes>::Value,
        _elt: <Self::CodegenCx as BackendTypes>::Value,
        _idx: u64,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn set_personality_fn(&mut self, _personality: <Self::CodegenCx as BackendTypes>::Value) {
        unimplemented!()
    }
    fn cleanup_landing_pad(
        &mut self,
        _pers_fn: <Self::CodegenCx as BackendTypes>::Value,
    ) -> (
        <Self::CodegenCx as BackendTypes>::Value,
        <Self::CodegenCx as BackendTypes>::Value,
        <Self::CodegenCx as BackendTypes>::Funclet,
    ) {
        unimplemented!()
    }
    fn filter_landing_pad(
        &mut self,
        _data: <Self::CodegenCx as BackendTypes>::Value,
        _filter: &[u64],
        _personality: <Self::CodegenCx as BackendTypes>::Value,
    ) -> (
        <Self::CodegenCx as BackendTypes>::Value,
        <Self::CodegenCx as BackendTypes>::Value,
        <Self::CodegenCx as BackendTypes>::Funclet,
    ) {
        unimplemented!()
    }
    fn resume(
        &mut self,
        _exn: <Self::CodegenCx as BackendTypes>::Value,
        _exn_data: <Self::CodegenCx as BackendTypes>::Value,
    ) {
        unimplemented!()
    }
    fn cleanup_pad(
        &mut self,
        _parent: Option<<Self::CodegenCx as BackendTypes>::Value>,
        _args: &[<Self::CodegenCx as BackendTypes>::Value],
    ) -> <Self::CodegenCx as BackendTypes>::Funclet {
        unimplemented!()
    }
    fn cleanup_ret(
        &mut self,
        _funclet: &<Self::CodegenCx as BackendTypes>::Funclet,
        _unwind: Option<<Self::CodegenCx as BackendTypes>::BasicBlock>,
    ) {
        unimplemented!()
    }
    fn catch_pad(
        &mut self,
        _parent: <Self::CodegenCx as BackendTypes>::Value,
        _args: &[<Self::CodegenCx as BackendTypes>::Value],
    ) -> <Self::CodegenCx as BackendTypes>::Funclet {
        unimplemented!()
    }
    fn catch_switch(
        &mut self,
        _parent: Option<<Self::CodegenCx as BackendTypes>::Value>,
        _unwind: Option<<Self::CodegenCx as BackendTypes>::BasicBlock>,
        _handler: <Self::CodegenCx as BackendTypes>::BasicBlock,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn atomic_cmpxchg(
        &mut self,
        _dst: <Self::CodegenCx as BackendTypes>::Value,
        _cmp: <Self::CodegenCx as BackendTypes>::Value,
        _src: <Self::CodegenCx as BackendTypes>::Value,
        _order: rustc_codegen_ssa::common::AtomicOrdering,
        _failure_order: rustc_codegen_ssa::common::AtomicOrdering,
        _weak: bool,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn atomic_rmw(
        &mut self,
        _op: rustc_codegen_ssa::common::AtomicRmwBinOp,
        _dst: <Self::CodegenCx as BackendTypes>::Value,
        _src: <Self::CodegenCx as BackendTypes>::Value,
        _order: rustc_codegen_ssa::common::AtomicOrdering,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn atomic_fence(
        &mut self,
        _order: rustc_codegen_ssa::common::AtomicOrdering,
        _scope: rustc_codegen_ssa::common::SynchronizationScope,
    ) {
        unimplemented!()
    }
    fn set_invariant_load(&mut self, _load: <Self::CodegenCx as BackendTypes>::Value) {
        unimplemented!()
    }
    fn lifetime_start(
        &mut self,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _size: rustc_target::abi::Size,
    ) {
        unimplemented!()
    }
    fn lifetime_end(
        &mut self,
        _ptr: <Self::CodegenCx as BackendTypes>::Value,
        _size: rustc_target::abi::Size,
    ) {
        unimplemented!()
    }
    fn call(
        &mut self,
        _ty: <Self::CodegenCx as BackendTypes>::Type,
        _fn_attrs: Option<&rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs>,
        _fn_abi: Option<&rustc_target::abi::call::FnAbi<'_, rustc_middle::ty::Ty<'_>>>,
        _llfn: <Self::CodegenCx as BackendTypes>::Value,
        _args: &[<Self::CodegenCx as BackendTypes>::Value],
        _funclet: Option<&<Self::CodegenCx as BackendTypes>::Funclet>,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn zext(
        &mut self,
        _val: <Self::CodegenCx as BackendTypes>::Value,
        _dest_ty: <Self::CodegenCx as BackendTypes>::Type,
    ) -> <Self::CodegenCx as BackendTypes>::Value {
        unimplemented!()
    }
    fn apply_attrs_to_cleanup_callsite(&mut self, _call: <Self::CodegenCx as BackendTypes>::Value) {
        unimplemented!()
    }
}
