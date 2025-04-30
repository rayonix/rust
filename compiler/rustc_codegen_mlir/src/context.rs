extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_target;

use melior::utility::register_all_dialects;
use rustc_abi::HasDataLayout;
use rustc_codegen_ssa::mir::operand::OperandRef;
use rustc_codegen_ssa::mir::{self, operand};
use rustc_codegen_ssa::traits::{
    self, AbiBuilderMethods, ArgAbiBuilderMethods, BaseTypeCodegenMethods, ConstCodegenMethods,
    CoverageInfoBuilderMethods, DebugInfoBuilderMethods, DebugInfoCodegenMethods,
    DerivedTypeCodegenMethods, IntrinsicCallBuilderMethods, LayoutTypeCodegenMethods,
    MiscCodegenMethods, StaticBuilderMethods, StaticCodegenMethods, TypeMembershipCodegenMethods,
    *,
};
use rustc_data_structures::fx::{self, FxHashMap};
use rustc_hir::def_id::{self, DefId};
use rustc_middle::mir::coverage::CoverageKind;
use rustc_middle::mir::mono::{Linkage, Visibility};
use rustc_middle::ty::layout::{
    FnAbiError, FnAbiOfHelpers, FnAbiRequest, LayoutError, MaybeResult,
};
use rustc_middle::ty::{self, Instance, Ty, TyCtxt};
use rustc_span::Span;
use rustc_target::callconv::{self, ArgAbi, FnAbi};

pub struct CodegenCx<'ml, 'tcx> {
    context: &'ml melior::Context,
    registry: &'ml melior::dialect::DialectRegistry,
    tcx: TyCtxt<'tcx>,
}

impl<'ml, 'tcx> CodegenCx<'ml, 'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>) -> Self {
        let mut registry = Box::new(melior::dialect::DialectRegistry::new());
        register_all_dialects(&registry);
        let context = Box::new(melior::Context::new());
        context.append_dialect_registry(&registry);
        context.load_all_available_dialects();

        // Leak the boxed values to extend their lifetimes
        let registry = Box::leak(registry);
        let context = Box::leak(context);

        Self { context, registry, tcx }
    }

    pub fn context(&self) -> &'ml melior::Context {
        self.context
    }

    pub fn registry(&self) -> &'ml melior::dialect::DialectRegistry {
        self.registry
    }
}

impl<'ml, 'tcx> BackendTypes for CodegenCx<'ml, 'tcx> {
    type Value = &'ml melior::ir::Value<'ml, 'ml>;
    type Metadata = &'ml melior::ir::Attribute<'ml>;
    type Function = &'ml melior::ir::Operation<'ml>;
    type BasicBlock = &'ml melior::ir::Block<'ml>;
    type Type = &'ml melior::ir::Type<'ml>;
    type Funclet = ();
    type DIScope = (); //TODO: Attribute needs to implement Hash. &'ml melior::ir::Attribute<'ml>;
    type DILocation = &'ml melior::ir::Location<'ml>;
    type DIVariable = &'ml melior::ir::Attribute<'ml>;
}

impl<'ml, 'tcx> PreDefineCodegenMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn predefine_fn(
        &self,
        instance: Instance<'tcx>,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    ) {
        unimplemented!("predefine_fn stub for CodegenCx");
    }
    fn predefine_static(
        &self,
        def_id: DefId,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    ) {
        unimplemented!("predefine_static stub for CodegenCx");
    }
}

impl<'ml, 'tcx> AsmCodegenMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn codegen_global_asm(
        &self,
        _template: &[rustc_ast::InlineAsmTemplatePiece],
        _operands: &[traits::GlobalAsmOperandRef<'tcx>],
        _options: rustc_ast::InlineAsmOptions,
        _line_spans: &[Span],
    ) {
        unimplemented!("codegen_global_asm stub for CodegenCx");
    }

    fn mangled_name(&self, _instance: Instance<'tcx>) -> String {
        unimplemented!("mangled_name stub for CodegenCx");
    }
}

impl<'ml, 'tcx> DebugInfoCodegenMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn create_vtable_debuginfo(
        &self,
        _ty: Ty<'tcx>,
        _trait_ref: Option<ty::ExistentialTraitRef<'tcx>>,
        _vtable: Self::Value,
    ) {
        unimplemented!("create_vtable_debuginfo for CodegenCx");
    }

    fn create_function_debug_context(
        &self,
        _instance: Instance<'tcx>,
        _fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        _llfn: Self::Function,
        _mir: &rustc_middle::mir::Body<'tcx>,
    ) -> Option<mir::debuginfo::FunctionDebugContext<'tcx, Self::DIScope, Self::DILocation>> {
        unimplemented!("create_function_debug_context for CodegenCx");
    }

    fn dbg_scope_fn(
        &self,
        _instance: Instance<'tcx>,
        _fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        _maybe_definition_llfn: Option<Self::Function>,
    ) -> Self::DIScope {
        unimplemented!("dbg_scope_fn for CodegenCx");
    }

    fn dbg_loc(
        &self,
        _scope: Self::DIScope,
        _inlined_at: Option<Self::DILocation>,
        _span: Span,
    ) -> Self::DILocation {
        unimplemented!("dbg_loc for CodegenCx");
    }

    fn extend_scope_to_file(
        &self,
        _scope_metadata: Self::DIScope,
        _file: &rustc_span::SourceFile,
    ) -> Self::DIScope {
        unimplemented!("extend_scope_to_file for CodegenCx");
    }

    fn debuginfo_finalize(&self) {
        unimplemented!("debuginfo_finalize for CodegenCx");
    }

    fn create_dbg_var(
        &self,
        _variable_name: rustc_span::Symbol,
        _variable_type: Ty<'tcx>,
        _scope_metadata: Self::DIScope,
        _variable_kind: mir::debuginfo::VariableKind,
        _span: Span,
    ) -> Self::DIVariable {
        unimplemented!("create_dbg_var for CodegenCx");
    }
}

impl<'ml, 'tcx> StaticCodegenMethods for CodegenCx<'ml, 'tcx> {
    fn static_addr_of(
        &self,
        _cv: Self::Value,
        _align: rustc_abi::Align,
        _kind: Option<&str>,
    ) -> Self::Value {
        unimplemented!("static_addr_of for CodegenCx");
    }
    fn codegen_static(&self, _def_id: DefId) {
        unimplemented!("codegen_static for CodegenCx");
    }
    fn add_used_global(&self, _global: Self::Value) {
        unimplemented!("add_used_global for CodegenCx");
    }
    fn add_compiler_used_global(&self, _global: Self::Value) {
        unimplemented!("add_compiler_used_global for CodegenCx");
    }
}

impl<'ml, 'tcx> ConstCodegenMethods for CodegenCx<'ml, 'tcx> {
    fn const_null(&self, _t: Self::Type) -> Self::Value {
        unimplemented!("const_null for CodegenCx")
    }
    fn const_undef(&self, _t: Self::Type) -> Self::Value {
        unimplemented!("const_undef for CodegenCx")
    }
    fn const_poison(&self, _t: Self::Type) -> Self::Value {
        unimplemented!("const_poison for CodegenCx")
    }
    fn const_bool(&self, _val: bool) -> Self::Value {
        unimplemented!("const_bool for CodegenCx")
    }
    fn const_i8(&self, _i: i8) -> Self::Value {
        unimplemented!("const_i8 for CodegenCx")
    }
    fn const_i16(&self, _i: i16) -> Self::Value {
        unimplemented!("const_i16 for CodegenCx")
    }
    fn const_i32(&self, _i: i32) -> Self::Value {
        unimplemented!("const_i32 for CodegenCx")
    }
    fn const_int(&self, _t: Self::Type, _i: i64) -> Self::Value {
        unimplemented!("const_int for CodegenCx")
    }
    fn const_u8(&self, _i: u8) -> Self::Value {
        unimplemented!("const_u8 for CodegenCx")
    }
    fn const_u32(&self, _i: u32) -> Self::Value {
        unimplemented!("const_u32 for CodegenCx")
    }
    fn const_u64(&self, _i: u64) -> Self::Value {
        unimplemented!("const_u64 for CodegenCx")
    }
    fn const_u128(&self, _i: u128) -> Self::Value {
        unimplemented!("const_u128 for CodegenCx")
    }
    fn const_usize(&self, _i: u64) -> Self::Value {
        unimplemented!("const_usize for CodegenCx")
    }
    fn const_uint(&self, _t: Self::Type, _i: u64) -> Self::Value {
        unimplemented!("const_uint for CodegenCx")
    }
    fn const_uint_big(&self, _t: Self::Type, _u: u128) -> Self::Value {
        unimplemented!("const_uint_big for CodegenCx")
    }
    fn const_real(&self, _t: Self::Type, _val: f64) -> Self::Value {
        unimplemented!("const_real for CodegenCx")
    }
    fn const_str(&self, _s: &str) -> (Self::Value, Self::Value) {
        unimplemented!("const_str for CodegenCx")
    }
    fn const_struct(&self, _elts: &[Self::Value], _packed: bool) -> Self::Value {
        unimplemented!("const_struct for CodegenCx")
    }
    fn const_vector(&self, _elts: &[Self::Value]) -> Self::Value {
        unimplemented!("const_vector for CodegenCx")
    }
    fn const_to_opt_uint(&self, _v: Self::Value) -> Option<u64> {
        unimplemented!("const_to_opt_uint for CodegenCx")
    }
    fn const_to_opt_u128(&self, _v: Self::Value, _sign_ext: bool) -> Option<u128> {
        unimplemented!("const_to_opt_u128 for CodegenCx")
    }
    fn const_data_from_alloc(
        &self,
        _alloc: rustc_middle::mir::interpret::ConstAllocation<'_>,
    ) -> Self::Value {
        unimplemented!("const_data_from_alloc for CodegenCx")
    }
    fn scalar_to_backend(
        &self,
        _cv: rustc_middle::mir::interpret::Scalar,
        _layout: rustc_abi::Scalar,
        _llty: Self::Type,
    ) -> Self::Value {
        unimplemented!("scalar_to_backend for CodegenCx")
    }
    fn const_ptr_byte_offset(&self, _val: Self::Value, _offset: rustc_abi::Size) -> Self::Value {
        unimplemented!("const_ptr_byte_offset for CodegenCx")
    }
}

impl<'ml, 'tcx> TypeMembershipCodegenMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn add_type_metadata(&self, _function: Self::Function, _typeid: String) {
        unimplemented!("add_type_metadata for CodegenCx");
    }
    fn set_type_metadata(&self, _function: Self::Function, _typeid: String) {
        unimplemented!("set_type_metadata for CodegenCx");
    }
    fn typeid_metadata(&self, _typeid: String) -> Option<Self::Metadata> {
        unimplemented!("typeid_metadata for CodegenCx");
    }
    fn add_kcfi_type_metadata(&self, _function: Self::Function, _typeid: u32) {
        unimplemented!("add_kcfi_type_metadata for CodegenCx");
    }
    fn set_kcfi_type_metadata(&self, _function: Self::Function, _typeid: u32) {
        unimplemented!("set_kcfi_type_metadata for CodegenCx");
    }
}

impl<'ml, 'tcx> LayoutTypeCodegenMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn backend_type(&self, _layout: ty::layout::TyAndLayout<'tcx>) -> Self::Type {
        unimplemented!("backend_type for CodegenCx");
    }
    fn cast_backend_type(&self, _ty: &callconv::CastTarget) -> Self::Type {
        unimplemented!("cast_backend_type for CodegenCx");
    }
    fn fn_decl_backend_type(&self, _fn_abi: &FnAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        unimplemented!("fn_decl_backend_type for CodegenCx");
    }
    fn fn_ptr_backend_type(&self, _fn_abi: &FnAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        unimplemented!("fn_ptr_backend_type for CodegenCx");
    }
    fn reg_backend_type(&self, _ty: &rustc_abi::Reg) -> Self::Type {
        unimplemented!("reg_backend_type for CodegenCx");
    }
    fn immediate_backend_type(&self, _layout: ty::layout::TyAndLayout<'tcx>) -> Self::Type {
        unimplemented!("immediate_backend_type for CodegenCx");
    }
    fn is_backend_immediate(&self, _layout: ty::layout::TyAndLayout<'tcx>) -> bool {
        unimplemented!("is_backend_immediate for CodegenCx");
    }
    fn is_backend_scalar_pair(&self, _layout: ty::layout::TyAndLayout<'tcx>) -> bool {
        unimplemented!("is_backend_scalar_pair for CodegenCx");
    }
    fn scalar_pair_element_backend_type(
        &self,
        _layout: ty::layout::TyAndLayout<'tcx>,
        _index: usize,
        _immediate: bool,
    ) -> Self::Type {
        unimplemented!("scalar_pair_element_backend_type for CodegenCx");
    }
}

impl<'ml, 'tcx> MiscCodegenMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn vtables(
        &self,
    ) -> &std::cell::RefCell<
        FxHashMap<(Ty<'tcx>, Option<ty::ExistentialTraitRef<'tcx>>), Self::Value>,
    > {
        unimplemented!("vtables for CodegenCx");
    }
    fn apply_vcall_visibility_metadata(
        &self,
        _ty: Ty<'tcx>,
        _poly_trait_ref: Option<ty::ExistentialTraitRef<'tcx>>,
        _vtable: Self::Value,
    ) {
        unimplemented!("apply_vcall_visibility_metadata for CodegenCx");
    }
    fn get_fn(&self, _instance: Instance<'tcx>) -> Self::Function {
        unimplemented!("get_fn for CodegenCx");
    }
    fn get_fn_addr(&self, _instance: Instance<'tcx>) -> Self::Value {
        unimplemented!("get_fn_addr for CodegenCx");
    }
    fn eh_personality(&self) -> Self::Value {
        unimplemented!("eh_personality for CodegenCx");
    }
    fn sess(&self) -> &rustc_session::Session {
        unimplemented!("sess for CodegenCx");
    }
    fn codegen_unit(&self) -> &'tcx rustc_middle::mir::mono::CodegenUnit<'tcx> {
        unimplemented!("codegen_unit for CodegenCx");
    }
    fn set_frame_pointer_type(&self, _llfn: Self::Function) {
        unimplemented!("set_frame_pointer_type for CodegenCx");
    }
    fn apply_target_cpu_attr(&self, _llfn: Self::Function) {
        unimplemented!("apply_target_cpu_attr for CodegenCx");
    }
    fn declare_c_main(&self, main_fn_ty: Self::Type) -> Option<Self::Function> {
        unimplemented!("declare_c_main for CodegenCx");
    }
}

impl<'ml, 'tcx> BaseTypeCodegenMethods for CodegenCx<'ml, 'tcx> {
    fn type_i8(&self) -> Self::Type {
        unimplemented!("type_i8 for CodegenCx")
    }
    fn type_i16(&self) -> Self::Type {
        unimplemented!("type_i16 for CodegenCx")
    }
    fn type_i32(&self) -> Self::Type {
        unimplemented!("type_i32 for CodegenCx")
    }
    fn type_i64(&self) -> Self::Type {
        unimplemented!("type_i64 for CodegenCx")
    }
    fn type_i128(&self) -> Self::Type {
        unimplemented!("type_i128 for CodegenCx")
    }
    fn type_isize(&self) -> Self::Type {
        unimplemented!("type_isize for CodegenCx")
    }
    fn type_f16(&self) -> Self::Type {
        unimplemented!("type_f16 for CodegenCx")
    }
    fn type_f32(&self) -> Self::Type {
        unimplemented!("type_f32 for CodegenCx")
    }
    fn type_f64(&self) -> Self::Type {
        unimplemented!("type_f64 for CodegenCx")
    }
    fn type_f128(&self) -> Self::Type {
        unimplemented!("type_f128 for CodegenCx")
    }
    fn type_array(&self, _ty: Self::Type, _len: u64) -> Self::Type {
        unimplemented!("type_array for CodegenCx")
    }
    fn type_func(&self, _args: &[Self::Type], _ret: Self::Type) -> Self::Type {
        unimplemented!("type_func for CodegenCx")
    }
    fn type_kind(&self, _ty: Self::Type) -> rustc_codegen_ssa::common::TypeKind {
        unimplemented!("type_kind for CodegenCx")
    }
    fn type_ptr(&self) -> Self::Type {
        unimplemented!("type_ptr for CodegenCx")
    }
    fn type_ptr_ext(&self, _address_space: rustc_abi::AddressSpace) -> Self::Type {
        unimplemented!("type_ptr_ext for CodegenCx")
    }
    fn element_type(&self, _ty: Self::Type) -> Self::Type {
        unimplemented!("element_type for CodegenCx")
    }
    fn vector_length(&self, _ty: Self::Type) -> usize {
        unimplemented!("vector_length for CodegenCx")
    }
    fn float_width(&self, _ty: Self::Type) -> usize {
        unimplemented!("float_width for CodegenCx")
    }
    fn int_width(&self, _ty: Self::Type) -> u64 {
        unimplemented!("int_width for CodegenCx")
    }
    fn val_ty(&self, _v: Self::Value) -> Self::Type {
        unimplemented!("val_ty for CodegenCx")
    }
}

impl<'ml, 'tcx> CoverageInfoBuilderMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn add_coverage(&mut self, instance: Instance<'tcx>, kind: &CoverageKind) {
        unimplemented!("add_coverage for CodegenCx");
    }

    fn init_coverage(&mut self, _instance: Instance<'tcx>) {}
}

impl<'ml, 'tcx> StaticBuilderMethods for CodegenCx<'ml, 'tcx> {
    fn get_static(&mut self, _def_id: DefId) -> Self::Value {
        unimplemented!("get_static for CodegenCx");
    }
}

impl<'ml, 'tcx> AbiBuilderMethods for CodegenCx<'ml, 'tcx> {
    fn get_param(&mut self, _index: usize) -> Self::Value {
        unimplemented!("get_param for CodegenCx");
    }
}

impl<'ml, 'tcx> ArgAbiBuilderMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn store_fn_arg(
        &mut self,
        _arg_abi: &ArgAbi<'tcx, Ty<'tcx>>,
        _idx: &mut usize,
        _dst: mir::place::PlaceRef<'tcx, Self::Value>,
    ) {
        unimplemented!("store_fn_arg for CodegenCx");
    }
    fn store_arg(
        &mut self,
        _arg_abi: &ArgAbi<'tcx, Ty<'tcx>>,
        _val: Self::Value,
        _dst: mir::place::PlaceRef<'tcx, Self::Value>,
    ) {
        unimplemented!("store_arg for CodegenCx");
    }
    fn arg_memory_ty(&self, _arg_abi: &ArgAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        unimplemented!("arg_memory_ty for CodegenCx");
    }
}

impl<'ml, 'tcx> IntrinsicCallBuilderMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn codegen_intrinsic_call(
        &mut self,
        _instance: Instance<'tcx>,
        _fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        _args: &[OperandRef<'tcx, Self::Value>],
        _llresult: Self::Value,
        _span: Span,
    ) -> Result<(), Instance<'tcx>> {
        unimplemented!("codegen_intrinsic_call for CodegenCx");
    }
    fn abort(&mut self) {
        unimplemented!("abort for CodegenCx");
    }
    fn assume(&mut self, _val: Self::Value) {
        unimplemented!("assume for CodegenCx");
    }
    fn expect(&mut self, _cond: Self::Value, _expected: bool) -> Self::Value {
        unimplemented!("expect for CodegenCx");
    }
    fn type_test(&mut self, _pointer: Self::Value, _typeid: Self::Metadata) -> Self::Value {
        unimplemented!("type_test for CodegenCx");
    }
    fn type_checked_load(
        &mut self,
        _llvtable: Self::Value,
        _vtable_byte_offset: u64,
        _typeid: Self::Metadata,
    ) -> Self::Value {
        unimplemented!("type_checked_load for CodegenCx");
    }
    fn va_start(&mut self, _val: Self::Value) -> Self::Value {
        unimplemented!("va_start for CodegenCx");
    }
    fn va_end(&mut self, _val: Self::Value) -> Self::Value {
        unimplemented!("va_end for CodegenCx");
    }
}

impl<'ml, 'tcx> DebugInfoBuilderMethods for CodegenCx<'ml, 'tcx> {
    fn dbg_var_addr(
        &mut self,
        _dbg_var: Self::DIVariable,
        _dbg_loc: Self::DILocation,
        _variable_alloca: Self::Value,
        _direct_offset: rustc_abi::Size,
        _indirect_offsets: &[rustc_abi::Size],
        _fragment: Option<std::ops::Range<rustc_abi::Size>>,
    ) {
        unimplemented!("dbg_var_addr for CodegenCx");
    }
    fn set_dbg_loc(&mut self, _dbg_loc: Self::DILocation) {
        unimplemented!("set_dbg_loc for CodegenCx");
    }
    fn clear_dbg_loc(&mut self) {
        unimplemented!("clear_dbg_loc for CodegenCx");
    }
    fn get_dbg_loc(&self) -> Option<Self::DILocation> {
        unimplemented!("get_dbg_loc for CodegenCx");
    }
    fn insert_reference_to_gdb_debug_scripts_section_global(&mut self) {
        unimplemented!("insert_reference_to_gdb_debug_scripts_section_global for CodegenCx");
    }
    fn set_var_name(&mut self, _value: Self::Value, _name: &str) {
        unimplemented!("set_var_name for CodegenCx");
    }
}

impl<'ml, 'tcx> FnAbiOfHelpers<'tcx> for CodegenCx<'ml, 'tcx> {
    type FnAbiOfResult = &'tcx FnAbi<'tcx, Ty<'tcx>>;
    fn handle_fn_abi_err(
        &self,
        err: FnAbiError<'tcx>,
        span: Span,
        fn_abi_request: FnAbiRequest<'tcx>,
    ) -> <Self::FnAbiOfResult as MaybeResult<&'tcx FnAbi<'tcx, Ty<'tcx>>>>::Error {
        unimplemented!("handle_fn_abi_err for CodegenCx");
    }
}

impl<'ml, 'tcx> ty::layout::LayoutOfHelpers<'tcx> for CodegenCx<'ml, 'tcx> {
    type LayoutOfResult = ty::layout::TyAndLayout<'tcx>;
    fn handle_layout_err(
        &self,
        _err: LayoutError<'tcx>,
        _span: Span,
        _ty: Ty<'tcx>,
    ) -> <Self::LayoutOfResult as MaybeResult<ty::layout::TyAndLayout<'tcx>>>::Error {
        unimplemented!("handle_layout_err for CodegenCx");
    }
    fn layout_tcx_at_span(&self) -> Span {
        unimplemented!("layout_tcx_at_span for CodegenCx");
    }
}

impl<'ml, 'tcx> ty::layout::HasTypingEnv<'tcx> for CodegenCx<'ml, 'tcx> {
    fn typing_env(&self) -> ty::TypingEnv<'tcx> {
        ty::TypingEnv::fully_monomorphized()
    }
}

impl<'ml, 'tcx> ty::layout::HasTyCtxt<'tcx> for CodegenCx<'ml, 'tcx> {
    fn tcx(&self) -> ty::TyCtxt<'tcx> {
        self.tcx
    }
}

impl<'ml, 'tcx> HasDataLayout for CodegenCx<'ml, 'tcx> {
    fn data_layout(&self) -> &rustc_abi::TargetDataLayout {
        self.tcx.data_layout()
    }
}
