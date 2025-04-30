#![allow(unused)]
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

use std::ops::Deref;

use melior::Context;
use rustc_abi::{Align, HasDataLayout, Scalar, Size, TargetDataLayout, WrappingRange};
use rustc_codegen_ssa::MemFlags;
use rustc_codegen_ssa::common::AtomicOrdering;
use rustc_codegen_ssa::mir::debuginfo::{FunctionDebugContext, VariableKind};
use rustc_codegen_ssa::mir::operand::OperandRef;
use rustc_codegen_ssa::mir::place::PlaceRef;
use rustc_codegen_ssa::traits::*;
use rustc_hir::def_id::DefId;
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs;
use rustc_middle::mir::coverage::CoverageKind;
use rustc_middle::ty::layout::{
    FnAbiError, FnAbiOfHelpers, FnAbiRequest, HasTypingEnv, LayoutError, LayoutOfHelpers,
    MaybeResult, TyAndLayout,
};
use rustc_middle::ty::{ExistentialTraitRef, Instance, ParamEnv, Ty, TyCtxt, TypingEnv};
use rustc_middle::{mir, ty};
use rustc_session::Session;
use rustc_span::symbol::Symbol;
use rustc_span::{DUMMY_SP, SourceFile, Span, source_map};
use rustc_target::callconv::{ArgAbi, FnAbi};
use rustc_target::spec::Target;

use crate::context::CodegenCx;

pub struct Builder<'ml, 'tcx> {
    cx: &'ml CodegenCx<'ml, 'tcx>,
    current_block: Option<<CodegenCx<'ml, 'tcx> as BackendTypes>::BasicBlock>,
    current_fn: Option<<CodegenCx<'ml, 'tcx> as BackendTypes>::Function>,
    current_span: Option<Span>,
}

impl<'ml, 'tcx> Deref for Builder<'ml, 'tcx> {
    type Target = CodegenCx<'ml, 'tcx>;
    fn deref(&self) -> &Self::Target {
        self.cx
    }
}

impl<'ml, 'tcx> BackendTypes for Builder<'ml, 'tcx> {
    type Value = <CodegenCx<'ml, 'tcx> as BackendTypes>::Value;
    type Function = <CodegenCx<'ml, 'tcx> as BackendTypes>::Function;
    type BasicBlock = <CodegenCx<'ml, 'tcx> as BackendTypes>::BasicBlock;
    type Type = <CodegenCx<'ml, 'tcx> as BackendTypes>::Type;
    type Funclet = <CodegenCx<'ml, 'tcx> as BackendTypes>::Funclet;

    // Metadata types
    type Metadata = <CodegenCx<'ml, 'tcx> as BackendTypes>::Metadata;

    // Debug info types
    type DIScope = <CodegenCx<'ml, 'tcx> as BackendTypes>::DIScope;
    type DILocation = <CodegenCx<'ml, 'tcx> as BackendTypes>::DILocation;
    type DIVariable = <CodegenCx<'ml, 'tcx> as BackendTypes>::DIVariable;
}

impl<'ml, 'tcx> BuilderMethods<'ml, 'tcx> for Builder<'ml, 'tcx> {
    type CodegenCx = CodegenCx<'ml, 'tcx>; // MLIR context for code generation operations

    fn build(cx: &'ml Self::CodegenCx, llbb: Self::BasicBlock) -> Self {
        Builder { cx, current_block: Some(llbb), current_fn: None, current_span: None }
    }

    fn cx(&self) -> &Self::CodegenCx {
        self.cx
    }

    fn llbb(&self) -> Self::BasicBlock {
        self.current_block.expect("no current block")
    }

    fn set_span(&mut self, span: rustc_span::Span) {
        self.current_span = Some(span);
    }

    fn append_block(
        cx: &'ml Self::CodegenCx,
        llfn: Self::Function,
        _name: &str,
    ) -> <Self::CodegenCx as BackendTypes>::BasicBlock {
        unimplemented!()
    }
    fn append_sibling_block(&mut self, name: &str) -> Self::BasicBlock {
        unimplemented!()
    }
    fn switch_to_block(&mut self, _bb: Self::BasicBlock) {
        unimplemented!()
    }
    fn ret_void(&mut self) {
        unimplemented!()
    }
    fn ret(&mut self, _value: Self::Value) {
        unimplemented!()
    }
    fn br(&mut self, _dest: Self::BasicBlock) {
        unimplemented!()
    }
    fn cond_br(
        &mut self,
        _cond: Self::Value,
        _then_bb: Self::BasicBlock,
        _else_bb: Self::BasicBlock,
    ) {
        unimplemented!()
    }
    fn switch(
        &mut self,
        v: Self::Value,
        else_llbb: Self::BasicBlock,
        cases: impl ExactSizeIterator<Item = (u128, Self::BasicBlock)>,
    ) {
        unimplemented!()
    }
    fn invoke(
        &mut self,
        _llty: Self::Type,
        _fn_attrs: Option<&CodegenFnAttrs>,
        _fn_abi: Option<&FnAbi<'tcx, Ty<'tcx>>>,
        _llfn: Self::Value,
        _args: &[Self::Value],
        _then: Self::BasicBlock,
        _catch: Self::BasicBlock,
        _funclet: Option<&Self::Funclet>,
        _instance: Option<Instance<'tcx>>,
    ) -> Self::Value {
        unimplemented!()
    }
    fn unreachable(&mut self) {
        unimplemented!()
    }
    fn add(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fadd(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fadd_fast(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fadd_algebraic(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn sub(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fsub(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fsub_fast(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fsub_algebraic(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn mul(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fmul(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fmul_fast(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fmul_algebraic(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn udiv(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn exactudiv(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn sdiv(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn exactsdiv(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fdiv(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fdiv_fast(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fdiv_algebraic(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn urem(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn srem(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn frem(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn frem_fast(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn frem_algebraic(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn shl(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn lshr(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn ashr(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn and(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn or(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn xor(&mut self, _lhs: Self::Value, _rhs: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn neg(&mut self, _value: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn fneg(&mut self, _value: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn not(&mut self, _value: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn checked_binop(
        &mut self,
        oop: OverflowOp,
        ty: Ty<'_>,
        lhs: Self::Value,
        rhs: Self::Value,
    ) -> (Self::Value, Self::Value) {
        unimplemented!()
    }
    fn from_immediate(&mut self, val: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn to_immediate_scalar(&mut self, val: Self::Value, scalar: Scalar) -> Self::Value {
        unimplemented!()
    }
    fn alloca(&mut self, size: Size, align: Align) -> Self::Value {
        unimplemented!()
    }
    fn dynamic_alloca(&mut self, _size: Self::Value, _align: Align) -> Self::Value {
        unimplemented!()
    }
    fn load(&mut self, _ty: Self::Type, _ptr: Self::Value, _align: Align) -> Self::Value {
        unimplemented!()
    }
    fn volatile_load(&mut self, ty: Self::Type, ptr: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn atomic_load(
        &mut self,
        ty: Self::Type,
        ptr: Self::Value,
        order: rustc_codegen_ssa::common::AtomicOrdering,
        size: Size,
    ) -> Self::Value {
        unimplemented!()
    }
    fn load_operand(
        &mut self,
        _place: rustc_codegen_ssa::mir::place::PlaceRef<'tcx, Self::Value>,
    ) -> rustc_codegen_ssa::mir::operand::OperandRef<'tcx, Self::Value> {
        unimplemented!()
    }
    fn write_operand_repeatedly(
        &mut self,
        elem: OperandRef<'tcx, Self::Value>,
        count: u64,
        dest: PlaceRef<'tcx, Self::Value>,
    ) {
        unimplemented!()
    }
    fn range_metadata(&mut self, load: Self::Value, range: WrappingRange) {
        unimplemented!()
    }
    fn nonnull_metadata(&mut self, _load: Self::Value) {
        unimplemented!()
    }
    fn store(&mut self, val: Self::Value, ptr: Self::Value, align: Align) -> Self::Value {
        unimplemented!()
    }
    fn store_with_flags(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        align: Align,
        flags: MemFlags,
    ) -> Self::Value {
        unimplemented!()
    }
    fn atomic_store(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        order: AtomicOrdering,
        size: Size,
    ) {
        unimplemented!()
    }
    fn gep(&mut self, _ty: Self::Type, _ptr: Self::Value, _indices: &[Self::Value]) -> Self::Value {
        unimplemented!()
    }
    fn inbounds_gep(
        &mut self,
        _ty: Self::Type,
        _ptr: Self::Value,
        _indices: &[Self::Value],
    ) -> Self::Value {
        unimplemented!()
    }
    fn trunc(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn sext(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn fptoui_sat(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn fptosi_sat(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn fptoui(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn fptosi(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn uitofp(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn sitofp(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn fptrunc(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn fpext(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn ptrtoint(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn inttoptr(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn bitcast(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn intcast(
        &mut self,
        _val: Self::Value,
        _dest_ty: Self::Type,
        _is_signed: bool,
    ) -> Self::Value {
        unimplemented!()
    }
    fn pointercast(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn icmp(
        &mut self,
        _op: rustc_codegen_ssa::common::IntPredicate,
        _lhs: Self::Value,
        _rhs: Self::Value,
    ) -> Self::Value {
        unimplemented!()
    }
    fn fcmp(
        &mut self,
        _op: rustc_codegen_ssa::common::RealPredicate,
        _lhs: Self::Value,
        _rhs: Self::Value,
    ) -> Self::Value {
        unimplemented!()
    }
    fn memcpy(
        &mut self,
        _dst: Self::Value,
        _dst_align: Align,
        _src: Self::Value,
        _src_align: Align,
        _size: Self::Value,
        _flags: rustc_codegen_ssa::MemFlags,
    ) {
        unimplemented!()
    }
    fn memmove(
        &mut self,
        _dst: Self::Value,
        _dst_align: Align,
        _src: Self::Value,
        _src_align: Align,
        _size: Self::Value,
        _flags: rustc_codegen_ssa::MemFlags,
    ) {
        unimplemented!()
    }
    fn memset(
        &mut self,
        _ptr: Self::Value,
        _fill_byte: Self::Value,
        _size: Self::Value,
        _align: Align,
        _flags: rustc_codegen_ssa::MemFlags,
    ) {
        unimplemented!()
    }
    fn select(
        &mut self,
        _cond: Self::Value,
        _then_val: Self::Value,
        _else_val: Self::Value,
    ) -> Self::Value {
        unimplemented!()
    }
    fn va_arg(&mut self, _list: Self::Value, _ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn extract_element(&mut self, _vec: Self::Value, _idx: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn vector_splat(&mut self, _num_elts: usize, _elt: Self::Value) -> Self::Value {
        unimplemented!()
    }
    fn extract_value(&mut self, _aggregate: Self::Value, _idx: u64) -> Self::Value {
        unimplemented!()
    }
    fn insert_value(
        &mut self,
        _aggregate: Self::Value,
        _elt: Self::Value,
        _idx: u64,
    ) -> Self::Value {
        unimplemented!()
    }
    fn set_personality_fn(&mut self, _personality: Self::Value) {
        unimplemented!()
    }
    fn cleanup_landing_pad(&mut self, _pers_fn: Self::Value) -> (Self::Value, Self::Value) {
        unimplemented!()
    }
    fn filter_landing_pad(&mut self, pers_fn: Self::Value) -> (Self::Value, Self::Value) {
        unimplemented!()
    }
    fn resume(&mut self, _exn: Self::Value, _exn_data: Self::Value) {
        unimplemented!()
    }
    fn cleanup_pad(
        &mut self,
        _parent: Option<Self::Value>,
        _args: &[Self::Value],
    ) -> Self::Funclet {
        unimplemented!()
    }
    fn cleanup_ret(&mut self, _funclet: &Self::Funclet, _unwind: Option<Self::BasicBlock>) {
        unimplemented!()
    }
    fn catch_pad(&mut self, _parent: Self::Value, _args: &[Self::Value]) -> Self::Funclet {
        unimplemented!()
    }
    fn catch_switch(
        &mut self,
        _parent: Option<Self::Value>,
        _unwind: Option<Self::BasicBlock>,
        _handlers: &[Self::BasicBlock],
    ) -> Self::Value {
        unimplemented!()
    }
    fn atomic_cmpxchg(
        &mut self,
        dst: Self::Value,
        cmp: Self::Value,
        src: Self::Value,
        order: AtomicOrdering,
        failure_order: AtomicOrdering,
        weak: bool,
    ) -> (Self::Value, Self::Value) {
        unimplemented!()
    }
    fn atomic_rmw(
        &mut self,
        _op: rustc_codegen_ssa::common::AtomicRmwBinOp,
        _dst: Self::Value,
        _src: Self::Value,
        _order: rustc_codegen_ssa::common::AtomicOrdering,
    ) -> Self::Value {
        unimplemented!()
    }
    fn atomic_fence(
        &mut self,
        _order: rustc_codegen_ssa::common::AtomicOrdering,
        _scope: rustc_codegen_ssa::common::SynchronizationScope,
    ) {
        unimplemented!()
    }
    fn set_invariant_load(&mut self, _load: Self::Value) {
        unimplemented!()
    }
    fn lifetime_start(&mut self, _ptr: Self::Value, _size: rustc_abi::Size) {
        unimplemented!()
    }
    fn lifetime_end(&mut self, _ptr: Self::Value, _size: rustc_abi::Size) {
        unimplemented!()
    }
    fn call(
        &mut self,
        llty: Self::Type,
        fn_attrs: Option<&CodegenFnAttrs>,
        fn_abi: Option<&FnAbi<'tcx, Ty<'tcx>>>,
        llfn: Self::Value,
        args: &[Self::Value],
        funclet: Option<&Self::Funclet>,
        instance: Option<Instance<'tcx>>,
    ) -> Self::Value {
        unimplemented!()
    }
    fn zext(&mut self, _val: Self::Value, _dest_ty: Self::Type) -> Self::Value {
        unimplemented!()
    }
    fn apply_attrs_to_cleanup_callsite(&mut self, _call: Self::Value) {
        unimplemented!()
    }
}

impl StaticBuilderMethods for Builder<'_, '_> {
    fn get_static(&mut self, def_id: DefId) -> Self::Value {
        todo!()
    }
}

// Implement inline assembly code generation (AsmBuilderMethods) for MLIR backend
impl<'ml, 'tcx> AsmBuilderMethods<'tcx> for Builder<'ml, 'tcx> {
    fn codegen_inline_asm(
        &mut self,
        template: &[rustc_ast::InlineAsmTemplatePiece],
        operands: &[InlineAsmOperandRef<'tcx, Self>],
        options: rustc_ast::InlineAsmOptions,
        line_spans: &[rustc_span::Span],
        instance: ty::Instance<'_>,
        dest: Option<Self::BasicBlock>,
        catch_funclet: Option<(Self::BasicBlock, Option<&Self::Funclet>)>,
    ) {
        // TODO: translate InlineAsmTemplatePiece and operands into MLIR InlineAsmOp
        // For now, emit as unimplemented, planning to hook into melior's operation builder
        let asm_text = template
            .iter()
            .map(|piece| match piece {
                rustc_ast::InlineAsmTemplatePiece::String(s) => s.to_string(),
                rustc_ast::InlineAsmTemplatePiece::Placeholder { operand_idx, .. } => {
                    format!("${}", operand_idx)
                }
            })
            .collect::<String>();
        unimplemented!("Inline assembly not yet supported in MLIR backend: {}", asm_text);
    }
}

impl<'tcx> IntrinsicCallBuilderMethods<'tcx> for Builder<'_, 'tcx> {
    fn codegen_intrinsic_call(
        &mut self,
        instance: ty::Instance<'tcx>,
        fn_abi: &rustc_target::callconv::FnAbi<'tcx, ty::Ty<'tcx>>,
        args: &[rustc_codegen_ssa::mir::operand::OperandRef<'tcx, Self::Value>],
        llresult: Self::Value,
        span: rustc_span::Span,
    ) -> Result<(), ty::Instance<'tcx>> {
        // Fallback to default intrinsic body for MLIR backend
        Err(instance)
    }

    fn abort(&mut self) {
        // No-op: MLIR backend does not support `abort` intrinsic directly
    }
    fn assume(&mut self, _val: Self::Value) {
        // No-op: MLIR backend does not emit `assume`
    }
    fn expect(&mut self, cond: Self::Value, _expected: bool) -> Self::Value {
        // Return the condition as-is
        cond
    }
    fn type_test(&mut self, pointer: Self::Value, _typeid: Self::Metadata) -> Self::Value {
        // Identity: MLIR backend does not implement type tests
        pointer
    }
    fn type_checked_load(
        &mut self,
        llvtable: Self::Value,
        _vtable_byte_offset: u64,
        _typeid: Self::Metadata,
    ) -> Self::Value {
        // Identity: MLIR backend does not implement type-checked loads
        llvtable
    }
    fn va_start(&mut self, val: Self::Value) -> Self::Value {
        // Identity: MLIR backend does not lower varargs
        val
    }
    fn va_end(&mut self, val: Self::Value) -> Self::Value {
        // Identity: MLIR backend does not lower varargs
        val
    }
}

impl AbiBuilderMethods for Builder<'_, '_> {
    fn get_param(&mut self, index: usize) -> Self::Value {
        todo!()
    }
}

impl<'tcx> FnAbiOfHelpers<'tcx> for Builder<'_, 'tcx> {
    type FnAbiOfResult = &'tcx FnAbi<'tcx, Ty<'tcx>>;

    fn handle_fn_abi_err(
        &self,
        err: FnAbiError<'tcx>,
        span: Span,
        fn_abi_request: FnAbiRequest<'tcx>,
    ) -> <Self::FnAbiOfResult as MaybeResult<&'tcx FnAbi<'tcx, Ty<'tcx>>>>::Error {
        todo!()
    }
}

impl<'tcx> ArgAbiBuilderMethods<'tcx> for Builder<'_, 'tcx> {
    fn store_fn_arg(
        &mut self,
        arg_abi: &ArgAbi<'tcx, Ty<'tcx>>,
        idx: &mut usize,
        dst: PlaceRef<'tcx, Self::Value>,
    ) {
        todo!()
    }

    fn store_arg(
        &mut self,
        arg_abi: &ArgAbi<'tcx, Ty<'tcx>>,
        val: Self::Value,
        dst: PlaceRef<'tcx, Self::Value>,
    ) {
        todo!()
    }

    fn arg_memory_ty(&self, arg_abi: &ArgAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        todo!()
    }
}

impl<'tcx> LayoutOfHelpers<'tcx> for Builder<'_, 'tcx> {
    type LayoutOfResult = TyAndLayout<'tcx>;

    fn handle_layout_err(
        &self,
        err: LayoutError<'tcx>,
        span: Span,
        ty: Ty<'tcx>,
    ) -> <Self::LayoutOfResult as MaybeResult<TyAndLayout<'tcx>>>::Error {
        todo!()
    }

    fn layout_tcx_at_span(&self) -> Span {
        DUMMY_SP
    }
}

impl DebugInfoBuilderMethods for Builder<'_, '_> {
    fn dbg_var_addr(
        &mut self,
        dbg_var: Self::DIVariable,
        dbg_loc: Self::DILocation,
        variable_alloca: Self::Value,
        direct_offset: Size,
        // NB: each offset implies a deref (i.e. they're steps in a pointer chain).
        indirect_offsets: &[Size],
        // Byte range in the `dbg_var` covered by this fragment,
        // if this is a fragment of a composite `DIVariable`.
        fragment: Option<std::ops::Range<Size>>,
    ) {
        todo!()
    }

    fn set_dbg_loc(&mut self, dbg_loc: Self::DILocation) {
        todo!()
    }

    fn clear_dbg_loc(&mut self) {
        todo!()
    }

    fn get_dbg_loc(&self) -> Option<Self::DILocation> {
        todo!()
    }

    fn insert_reference_to_gdb_debug_scripts_section_global(&mut self) {
        todo!()
    }

    fn set_var_name(&mut self, value: Self::Value, name: &str) {
        todo!()
    }
}

impl<'tcx> DebugInfoCodegenMethods<'tcx> for Builder<'_, 'tcx> {
    fn create_vtable_debuginfo(
        &self,
        ty: Ty<'tcx>,
        trait_ref: Option<ExistentialTraitRef<'tcx>>,
        vtable: Self::Value,
    ) {
        unimplemented!()
    }

    fn dbg_scope_fn(
        &self,
        instance: Instance<'tcx>,
        fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        maybe_definition_llfn: Option<Self::Function>,
    ) -> Self::DIScope {
        unimplemented!()
    }

    fn dbg_loc(
        &self,
        scope: Self::DIScope,
        inlined_at: Option<Self::DILocation>,
        span: Span,
    ) -> Self::DILocation {
        unimplemented!()
    }

    fn extend_scope_to_file(
        &self,
        scope_metadata: Self::DIScope,
        file: &SourceFile,
    ) -> Self::DIScope {
        unimplemented!()
    }

    fn debuginfo_finalize(&self) {
        unimplemented!()
    }

    fn create_dbg_var(
        &self,
        variable_name: Symbol,
        variable_type: Ty<'tcx>,
        scope_metadata: Self::DIScope,
        variable_kind: VariableKind,
        span: Span,
    ) -> Self::DIVariable {
        unimplemented!()
    }

    fn create_function_debug_context(
        &self,
        instance: Instance<'tcx>,
        fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        llfn: Self::Function,
        mir: &mir::Body<'tcx>,
    ) -> Option<FunctionDebugContext<'tcx, Self::DIScope, Self::DILocation>> {
        todo!()
    }
}

impl<'tcx> MiscCodegenMethods<'tcx> for Builder<'_, 'tcx> {
    fn vtables(
        &self,
    ) -> &std::cell::RefCell<
        rustc_data_structures::fx::FxHashMap<
            (ty::Ty<'tcx>, Option<ty::ExistentialTraitRef<'tcx>>),
            Self::Value,
        >,
    > {
        unimplemented!()
    }

    fn apply_vcall_visibility_metadata(
        &self,
        _ty: ty::Ty<'tcx>,
        _poly_trait_ref: Option<ty::ExistentialTraitRef<'tcx>>,
        _vtable: Self::Value,
    ) {
        unimplemented!()
    }

    fn get_fn(&self, _instance: ty::Instance<'tcx>) -> Self::Function {
        unimplemented!()
    }

    fn get_fn_addr(&self, _instance: ty::Instance<'tcx>) -> Self::Value {
        unimplemented!()
    }

    fn eh_personality(&self) -> Self::Value {
        unimplemented!()
    }

    fn sess(&self) -> &Session {
        unimplemented!()
    }

    fn codegen_unit(&self) -> &'tcx rustc_middle::mir::mono::CodegenUnit<'tcx> {
        unimplemented!()
    }

    fn set_frame_pointer_type(&self, _llfn: Self::Function) {
        unimplemented!()
    }

    fn apply_target_cpu_attr(&self, _llfn: Self::Function) {
        unimplemented!()
    }

    fn declare_c_main(&self, main_fn_ty: Self::Type) -> Option<Self::Function> {
        unimplemented!()
    }
}

impl<'tcx> CoverageInfoBuilderMethods<'tcx> for Builder<'_, 'tcx> {
    fn add_coverage(&mut self, instance: Instance<'tcx>, kind: &CoverageKind) {
        unimplemented!()
    }
}

impl<'tcx> HasTypingEnv<'tcx> for Builder<'_, 'tcx> {
    fn typing_env<'a>(&'a self) -> TypingEnv<'tcx> {
        unimplemented!("HasTypingEnv::typing_env for Builder")
    }

    fn param_env(&self) -> ParamEnv<'tcx> {
        unimplemented!("HasTypingEnv::param_env for Builder")
    }
}

impl<'tcx> ty::layout::HasTyCtxt<'tcx> for Builder<'_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        unimplemented!("HasTyCtxt for Builder: return TyCtxt")
    }
}

impl HasDataLayout for Builder<'_, '_> {
    fn data_layout(&self) -> &TargetDataLayout {
        unimplemented!("HasDataLayout for Builder: return TargetDataLayout")
    }
}
