#[macro_use]
extern crate tracing;

use crate::abi::MlirType;
use crate::callee::get_fn;

use mlir_sys;

use rustc_ast::expand::allocator::AllocatorKind;
use rustc_ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_codegen_ssa::back::lto::{LtoModuleCodegen, SerializedModule, ThinModule};
use rustc_codegen_ssa::back::write::{
    CodegenContext, FatLTOInput, ModuleConfig, TargetMachineFactoryFn,
};
use rustc_codegen_ssa::base::maybe_create_entry_wrapper;
use rustc_codegen_ssa::common::{IntPredicate, RealPredicate, SynchronizationScope, TypeKind};
use rustc_codegen_ssa::mir::debuginfo::{DebugScope, FunctionDebugContext, VariableKind};
use rustc_codegen_ssa::mir::operand::OperandRef;
use rustc_codegen_ssa::mir::place::PlaceRef;
use rustc_codegen_ssa::mono_item::MonoItemExt;
use rustc_codegen_ssa::target_features::supported_target_features;
use rustc_codegen_ssa::traits::*;
use rustc_codegen_ssa::{MemFlags, CodegenResults, CompiledModule, ModuleCodegen, ModuleKind};
use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::tagged_ptr::Pointer;
use rustc_errors::{DiagnosticMessage, ErrorGuaranteed, FatalError, Handler, SubdiagnosticMessage};
use rustc_hir::def_id::DefId;
use rustc_index::vec::IndexVec;
use rustc_macros::fluent_messages;
use rustc_metadata::EncodedMetadata;
use rustc_middle::bug;
use rustc_middle::dep_graph;
use rustc_middle::dep_graph::{WorkProduct, WorkProductId};
use rustc_middle::mir;
use rustc_middle::mir::coverage::{
    CodeRegion, CounterValueReference, ExpressionOperandId, InjectedExpressionId, Op,
};
use rustc_middle::mir::interpret::{ConstAllocation, Scalar};
use rustc_middle::mir::mono::{CodegenUnit, Linkage, Visibility};
use rustc_middle::ty::layout::{
    FnAbiError, FnAbiOfHelpers, FnAbiRequest, HasParamEnv, HasTyCtxt, LayoutError, LayoutOfHelpers,
    TyAndLayout,
};
use rustc_middle::ty::query::Providers;
use rustc_middle::ty::{self, Instance, ParamEnv, TypeVisitableExt};
use rustc_middle::ty::{PolyExistentialTraitRef, Ty, TyCtxt};
use rustc_middle::ty::layout::{FnAbiOf, LayoutOf};
use rustc_session::config::{Lto, OptLevel, OutputFilenames, PrintRequest, DebugInfo};
use rustc_session::Session;
use rustc_span::{self, BytePos, Pos, Symbol, SourceFile, SourceFileAndLine, SourceFileHash, Span};
use rustc_target::abi::call::{ArgAbi, CastTarget, FnAbi, Reg};
use rustc_target::abi::{
    Abi, AddressSpace, Align, HasDataLayout, Size, TargetDataLayout, WrappingRange,
};
use rustc_target::spec::{HasTargetSpec, Target};

use std::any::Any;
use std::cell::RefCell;
use std::ffi::CString;
use std::ops::Deref;
use std::ops::Range;
use std::os::raw::c_char;
use std::sync::Arc;
use std::time::Instant;

mod base;
mod abi;
mod callee;

fluent_messages! { "../locales/en-US.ftl" }

pub struct Funclet {}

pub struct CodegenCx<'ml, 'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub codegen_unit: &'tcx CodegenUnit<'tcx>,
    pub mlirmod: &'ml mlir_sys::MlirModule,
    pub mlirctx: &'ml mlir_sys::MlirContext,
    pub instances: RefCell<FxHashMap<Instance<'tcx>, <Self as BackendTypes>::Function>>,
}

impl<'ml, 'tcx> CodegenCx<'ml, 'tcx> {
    pub(crate) fn new(
        tcx: TyCtxt<'tcx>,
        codegen_unit: &'tcx CodegenUnit<'tcx>,
        mlir_module: &'ml crate::ModuleMlir,
    ) -> Self {
        unsafe {
            let registry = mlir_sys::mlirDialectRegistryCreate();
            mlir_sys::mlirRegisterAllDialects(registry);
            mlir_sys::mlirContextAppendDialectRegistry(mlir_module.mlcx, registry);
            mlir_sys::mlirDialectRegistryDestroy(registry);
            mlir_sys::mlirContextLoadAllAvailableDialects(mlir_module.mlcx);
        }
        CodegenCx {
            tcx,
            codegen_unit,
            mlirmod: &mlir_module.mlmod_raw,
            mlirctx: &mlir_module.mlcx,
            instances: RefCell::new(FxHashMap::default()),
        }
    }

    pub fn declare_fn(&self, name: &str, fn_abi: &FnAbi<'tcx, Ty<'tcx>>) -> <Self as BackendTypes>::Function {
        debug!("declare_fn(name={:?}, fn_abi={:?})", name, fn_abi);

        let fntype: mlir_sys::MlirType = fn_abi.mlir_type(self).try_into().unwrap();
        debug!("declare_fn(name={:?}, ty={:?})", name, fntype);
        unsafe {
            let loc = mlir_sys::mlirLocationUnknownGet(*self.mlirctx);
            let num_inputs = mlir_sys::mlirFunctionTypeGetNumInputs(fntype);
            let num_results = mlir_sys::mlirFunctionTypeGetNumResults(fntype);
            let mut inputs = Vec::new();
            let mut locs = Vec::new();
            for i in 0..num_inputs {
                let input = mlir_sys::mlirFunctionTypeGetInput(fntype, i);
                let loc = mlir_sys::mlirLocationUnknownGet(*self.mlirctx);
                inputs.push(input);
                locs.push(loc);
                debug!("declare_fn: input {}: {:?}", i, input);
            }
            // zip up the operands with their location to create the arguments to a block
            let block = mlir_sys::mlirBlockCreate(num_inputs, inputs.as_ptr(), locs.as_ptr());
            let mut results = Vec::new();
            for i in 0..num_results {
                let result = mlir_sys::mlirFunctionTypeGetResult(fntype, i);
                results.push(result);
                debug!("declare_fn: arg {}: {:?}", i, result);
            }
            let func_str = CString::new("func.func").unwrap();
            let op_func_str = mlir_sys::mlirStringRefCreateFromCString(func_str.as_ptr());
            let mut op_state = mlir_sys::mlirOperationStateGet(
                op_func_str,
                loc
            );
            let mlfn = mlir_sys::mlirOperationCreate(&mut op_state);

            if self.tcx.sess.is_sanitizer_cfi_enabled() {
                unreachable!("sanitizer cfi not supported yet");
            }

            if self.tcx.sess.is_sanitizer_kcfi_enabled() {
                unreachable!("sanitizer kcfi not supported yet");
            }

            mlfn
        }
    }

}

impl<'ml> BackendTypes for CodegenCx<'ml, '_> {
    type Value = mlir_sys::MlirValue;
    type Function = mlir_sys::MlirOperation;
    type BasicBlock = mlir_sys::MlirBlock;
    type Type = mlir_sys::MlirType;
    type Funclet = Funclet;

    type DIScope = mlir_sys::MlirAttribute;
    type DILocation = mlir_sys::MlirAttribute;
    type DIVariable = mlir_sys::MlirAttribute;
}

impl<'ml, 'tcx> TypeMembershipMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn set_type_metadata(&self, function: Self::Function, typeid: String) {
        todo!();
    }
    fn typeid_metadata(&self, typeid: String) -> Self::Value {
        todo!();
    }
    fn set_kcfi_type_metadata(&self, function: Self::Function, typeid: u32) {
        todo!();
    }
}

impl<'ml, 'tcx> LayoutTypeMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn backend_type(&self, layout: TyAndLayout<'tcx>) -> Self::Type {
        todo!();
    }
    fn cast_backend_type(&self, ty: &CastTarget) -> Self::Type {
        todo!();
    }
    fn fn_decl_backend_type(&self, fn_abi: &FnAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        todo!();
    }
    fn fn_ptr_backend_type(&self, fn_abi: &FnAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        todo!();
    }
    fn reg_backend_type(&self, ty: &Reg) -> Self::Type {
        todo!();
    }
    fn immediate_backend_type(&self, layout: TyAndLayout<'tcx>) -> Self::Type {
        todo!();
    }
    fn is_backend_immediate(&self, layout: TyAndLayout<'tcx>) -> bool {
        todo!();
    }
    fn is_backend_scalar_pair(&self, layout: TyAndLayout<'tcx>) -> bool {
        todo!();
    }
    fn backend_field_index(&self, layout: TyAndLayout<'tcx>, index: usize) -> u64 {
        todo!();
    }
    fn scalar_pair_element_backend_type(
        &self,
        layout: TyAndLayout<'tcx>,
        index: usize,
        immediate: bool,
    ) -> Self::Type {
        todo!();
    }
}

impl<'ml, 'tcx> BaseTypeMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn type_i1(&self) -> Self::Type {
        unsafe {
            mlir_sys::mlirIntegerTypeGet(*self.mlirctx, 1)
        }
    }
    fn type_i8(&self) -> Self::Type {
        unsafe {
            mlir_sys::mlirIntegerTypeSignedGet(*self.mlirctx, 8)
        }
    }
    fn type_i16(&self) -> Self::Type {
        unsafe {
            mlir_sys::mlirIntegerTypeSignedGet(*self.mlirctx, 16)
        }
    }
    fn type_i32(&self) -> Self::Type {
        unsafe {
            mlir_sys::mlirIntegerTypeSignedGet(*self.mlirctx, 32)
        }
    }
    fn type_i64(&self) -> Self::Type {
        unsafe {
            mlir_sys::mlirIntegerTypeSignedGet(*self.mlirctx, 64)
        }
    }
    fn type_i128(&self) -> Self::Type {
        unsafe {
            mlir_sys::mlirIntegerTypeSignedGet(*self.mlirctx, 128)
        }
    }
    fn type_isize(&self) -> Self::Type {
        unsafe {
            mlir_sys::mlirIntegerTypeSignedGet(*self.mlirctx, self.tcx.data_layout.pointer_size.bits() as u32)
        }
    }

    fn type_f32(&self) -> Self::Type {
        unsafe {
            mlir_sys::mlirF32TypeGet(*self.mlirctx)
        }
    }
    fn type_f64(&self) -> Self::Type {
        unsafe {
            mlir_sys::mlirF64TypeGet(*self.mlirctx)
        }
    }

    fn type_array(&self, ty: Self::Type, len: u64) -> Self::Type {
        unsafe {
            mlir_sys::mlirRankedTensorTypeGet(
                1 as isize,
                &len as *const u64 as *const i64,
                ty,
                mlir_sys::mlirAttributeGetNull(),
            )
        }
    }
    fn type_func(&self, args: &[Self::Type], ret: Self::Type) -> Self::Type {
        unsafe {
            mlir_sys::mlirFunctionTypeGet(
                *self.mlirctx,
                args.len() as isize,
                args.as_ptr(),
                1,
                &ret as *const Self::Type
            )
        }
    }
    fn type_struct(&self, els: &[Self::Type], packed: bool) -> Self::Type {
        unsafe {
            mlir_sys::mlirTupleTypeGet(
                *self.mlirctx,
                els.len() as isize,
                els.as_ptr(),
            )
        }
    }
    fn type_kind(&self, ty: Self::Type) -> TypeKind {
        unsafe {
            if mlir_sys::mlirTypeIsAInteger(ty) {
                TypeKind::Integer
            } else if mlir_sys::mlirTypeIsAF16(ty) || mlir_sys::mlirTypeIsAF32(ty) || mlir_sys::mlirTypeIsAF64(ty) {
                TypeKind::Float
            } else if mlir_sys::mlirTypeIsATuple(ty) {
                TypeKind::Struct
            } else if mlir_sys::mlirTypeIsAFunction(ty) {
                TypeKind::Function
            } else if mlir_sys::mlirTypeIsAMemRef(ty) {
                TypeKind::Pointer
            } else if mlir_sys::mlirTypeIsARankedTensor(ty) {
                TypeKind::Array
            } else {
                TypeKind::Void
            }
        }
    }
    fn type_ptr_to(&self, ty: Self::Type) -> Self::Type {
        unsafe {
            let attr_ty_str = mlir_sys::mlirStringRefCreateFromCString(CString::new(AddressSpace::DATA.0.to_string()).unwrap().as_ptr());
            let attr = mlir_sys::mlirAttributeParseGet(*self.mlirctx, attr_ty_str);
            mlir_sys::mlirUnrankedMemRefTypeGet(ty, attr)
        }
    }
    fn type_ptr_to_ext(&self, ty: Self::Type, address_space: AddressSpace) -> Self::Type {
        unsafe {
            let attr_ty_str = mlir_sys::mlirStringRefCreateFromCString(CString::new(address_space.0.to_string()).unwrap().as_ptr());
            let attr = mlir_sys::mlirAttributeParseGet(*self.mlirctx, attr_ty_str);
            mlir_sys::mlirUnrankedMemRefTypeGet(ty, attr)
        }
    }
    fn element_type(&self, ty: Self::Type) -> Self::Type {
        match self.type_kind(ty) {
            TypeKind::Array | TypeKind::Vector => unsafe {
                mlir_sys::mlirShapedTypeGetElementType(ty)
            },
            TypeKind::Pointer => bug!("element_type is not supported for opaque pointers"),
            other => bug!("element_type called on unsupported type {:?}", other),
        }
    }

    /// Returns the number of elements in `self` if it is a LLVM vector type.
    fn vector_length(&self, ty: Self::Type) -> usize {
        todo!();
    }

    fn float_width(&self, ty: Self::Type) -> usize {
        todo!();
    }

    /// Retrieves the bit width of the integer type `self`.
    fn int_width(&self, ty: Self::Type) -> u64 {
        todo!();
    }

    fn val_ty(&self, v: Self::Value) -> Self::Type {
        todo!();
    }
}

impl<'ml, 'tcx> MiscMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn vtables(
        &self,
    ) -> &RefCell<FxHashMap<(Ty<'tcx>, Option<PolyExistentialTraitRef<'tcx>>), Self::Value>> {
        todo!();
    }
    fn check_overflow(&self) -> bool {
        todo!();
    }
    fn get_fn(&self, instance: Instance<'tcx>) -> Self::Function {
        get_fn(self, instance)
    }
    fn get_fn_addr(&self, instance: Instance<'tcx>) -> Self::Value {
        todo!();
    }
    fn eh_personality(&self) -> Self::Value {
        todo!();
    }
    fn sess(&self) -> &Session {
        todo!();
    }
    fn codegen_unit(&self) -> &'tcx CodegenUnit<'tcx> {
        todo!();
    }
    fn set_frame_pointer_type(&self, llfn: Self::Function) {
        todo!();
    }
    fn apply_target_cpu_attr(&self, llfn: Self::Function) {
        todo!();
    }
    /// Declares the extern "C" main function for the entry point. Returns None if the symbol already exists.
    fn declare_c_main(&self, fn_type: Self::Type) -> Option<Self::Function> {
        todo!();
    }
}

impl<'ml, 'tcx> ConstMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    // Constant constructors
    fn const_null(&self, t: Self::Type) -> Self::Value {
        todo!();
    }
    fn const_undef(&self, t: Self::Type) -> Self::Value {
        todo!();
    }
    fn const_int(&self, t: Self::Type, i: i64) -> Self::Value {
        todo!();
    }
    fn const_uint(&self, t: Self::Type, i: u64) -> Self::Value {
        todo!();
    }
    fn const_uint_big(&self, t: Self::Type, u: u128) -> Self::Value {
        todo!();
    }
    fn const_bool(&self, val: bool) -> Self::Value {
        todo!();
    }
    fn const_i16(&self, i: i16) -> Self::Value {
        todo!();
    }
    fn const_i32(&self, i: i32) -> Self::Value {
        todo!();
    }
    fn const_u32(&self, i: u32) -> Self::Value {
        todo!();
    }
    fn const_u64(&self, i: u64) -> Self::Value {
        todo!();
    }
    fn const_usize(&self, i: u64) -> Self::Value {
        todo!();
    }
    fn const_u8(&self, i: u8) -> Self::Value {
        todo!();
    }
    fn const_real(&self, t: Self::Type, val: f64) -> Self::Value {
        todo!();
    }

    fn const_str(&self, s: &str) -> (Self::Value, Self::Value) {
        todo!();
    }
    fn const_struct(&self, elts: &[Self::Value], packed: bool) -> Self::Value {
        todo!();
    }

    fn const_to_opt_uint(&self, v: Self::Value) -> Option<u64> {
        todo!();
    }
    fn const_to_opt_u128(&self, v: Self::Value, sign_ext: bool) -> Option<u128> {
        todo!();
    }

    fn const_data_from_alloc(&self, alloc: ConstAllocation<'tcx>) -> Self::Value {
        todo!();
    }

    fn scalar_to_backend(&self, cv: Scalar, layout: rustc_target::abi::Scalar, llty: Self::Type) -> Self::Value {
        todo!();
    }

    fn from_const_alloc(
        &self,
        layout: TyAndLayout<'tcx>,
        alloc: ConstAllocation<'tcx>,
        offset: Size,
    ) -> PlaceRef<'tcx, Self::Value> {
        todo!();
    }

    fn const_ptrcast(&self, val: Self::Value, ty: Self::Type) -> Self::Value {
        todo!();
    }
}

impl<'ml, 'tcx> StaticMethods for CodegenCx<'ml, 'tcx> {
    fn static_addr_of(&self, cv: Self::Value, align: Align, kind: Option<&str>) -> Self::Value {
        todo!();
    }
    fn codegen_static(&self, def_id: DefId, is_mutable: bool) {
        todo!();
    }

    /// Mark the given global value as "used", to prevent the compiler and linker from potentially
    /// removing a static variable that may otherwise appear unused.
    fn add_used_global(&self, global: Self::Value) {
        todo!();
    }

    /// Same as add_used_global(), but only prevent the compiler from potentially removing an
    /// otherwise unused symbol. The linker is still permitted to drop it.
    ///
    /// This corresponds to the documented semantics of the `#[used]` attribute, although
    /// on some targets (non-ELF), we may use `add_used_global` for `#[used]` statics
    /// instead.
    fn add_compiler_used_global(&self, global: Self::Value) {
        todo!();
    }
}

impl<'ml, 'tcx> CoverageInfoMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn coverageinfo_finalize(&self) {
        todo!();
    }

    /// Codegen a small function that will never be called, with one counter
    /// that will never be incremented, that gives LLVM coverage tools a
    /// function definition it needs in order to resolve coverage map references
    /// to unused functions. This is necessary so unused functions will appear
    /// as uncovered (coverage execution count `0`) in LLVM coverage reports.
    fn define_unused_fn(&self, def_id: DefId) {
        todo!();
    }

    /// For LLVM codegen, returns a function-specific `Value` for a global
    /// string, to hold the function name passed to LLVM intrinsic
    /// `instrprof.increment()`. The `Value` is only created once per instance.
    /// Multiple invocations with the same instance return the same `Value`.
    fn get_pgo_func_name_var(&self, instance: Instance<'tcx>) -> Self::Value {
        todo!();
    }
}

impl<'ml, 'tcx> DebugInfoMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn create_vtable_debuginfo(
        &self,
        ty: Ty<'tcx>,
        trait_ref: Option<PolyExistentialTraitRef<'tcx>>,
        vtable: Self::Value,
    ) {
        todo!();
    }

    /// Creates the function-specific debug context.
    ///
    /// Returns the FunctionDebugContext for the function which holds state needed
    /// for debug info creation, if it is enabled.
    fn create_function_debug_context(
        &self,
        instance: Instance<'tcx>,
        fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        llfn: Self::Function,
        mir: &mir::Body<'tcx>,
    ) -> Option<FunctionDebugContext<Self::DIScope, Self::DILocation>> {
        None
    }

    // FIXME(eddyb) find a common convention for all of the debuginfo-related
    // names (choose between `dbg`, `debug`, `debuginfo`, `debug_info` etc.).
    fn dbg_scope_fn(
        &self,
        instance: Instance<'tcx>,
        fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        maybe_definition_llfn: Option<Self::Function>,
    ) -> Self::DIScope {
        todo!();
    }

    fn dbg_loc(
        &self,
        scope: Self::DIScope,
        inlined_at: Option<Self::DILocation>,
        span: Span,
    ) -> Self::DILocation {
        todo!();
    }

    fn extend_scope_to_file(
        &self,
        scope_metadata: Self::DIScope,
        file: &SourceFile,
    ) -> Self::DIScope {
        todo!();
    }
    fn debuginfo_finalize(&self) {
        todo!();
    }

    // FIXME(eddyb) find a common convention for all of the debuginfo-related
    // names (choose between `dbg`, `debug`, `debuginfo`, `debug_info` etc.).
    fn create_dbg_var(
        &self,
        variable_name: Symbol,
        variable_type: Ty<'tcx>,
        scope_metadata: Self::DIScope,
        variable_kind: VariableKind,
        span: Span,
    ) -> Self::DIVariable {
        todo!();
    }
}

impl<'ml, 'tcx> AsmMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn codegen_global_asm(
        &self,
        template: &[InlineAsmTemplatePiece],
        operands: &[GlobalAsmOperandRef<'tcx>],
        options: InlineAsmOptions,
        line_spans: &[Span],
    ) {
        todo!();
    }
}

impl<'ml, 'tcx> PreDefineMethods<'tcx> for CodegenCx<'ml, 'tcx> {
    fn predefine_static(
        &self,
        def_id: DefId,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    ) {
        todo!();
    }
    fn predefine_fn(
        &self,
        instance: Instance<'tcx>,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    ) {
        assert!(!instance.substs.needs_infer());

        // TODO: Does MLIR really not need declarations?

        debug!("predefine_fn: instance = {:?}", instance);

    }
}

impl<'ml, 'tcx> HasTargetSpec for CodegenCx<'ml, 'tcx> {
    fn target_spec(&self) -> &Target {
        todo!();
    }
}

impl HasDataLayout for CodegenCx<'_, '_> {
    #[inline]
    fn data_layout(&self) -> &TargetDataLayout {
        &self.tcx.data_layout
    }
}

impl<'tcx, 'll> HasParamEnv<'tcx> for CodegenCx<'ll, 'tcx> {
    fn param_env(&self) -> ParamEnv<'tcx> {
        ParamEnv::reveal_all()
    }
}

impl<'tcx> LayoutOfHelpers<'tcx> for CodegenCx<'_, 'tcx> {
    type LayoutOfResult = TyAndLayout<'tcx>;

    #[inline]
    fn handle_layout_err(&self, err: LayoutError<'tcx>, span: Span, ty: Ty<'tcx>) -> ! {
        todo!();
    }
}

impl<'tcx> FnAbiOfHelpers<'tcx> for CodegenCx<'_, 'tcx> {
    type FnAbiOfResult = &'tcx FnAbi<'tcx, Ty<'tcx>>;

    #[inline]
    fn handle_fn_abi_err(
        &self,
        err: FnAbiError<'tcx>,
        span: Span,
        fn_abi_request: FnAbiRequest<'tcx>,
    ) -> ! {
        todo!();
    }
}

impl<'ml, 'tcx> HasTyCtxt<'tcx> for CodegenCx<'ml, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
}

pub struct ModuleBuffer {}

impl ModuleBufferMethods for ModuleBuffer {
    fn data(&self) -> &[u8] {
        todo!();
    }
}

pub struct ThinData {}

pub struct ThinBuffer {}

impl ThinBufferMethods for ThinBuffer {
    fn data(&self) -> &[u8] {
        todo!();
    }
}

pub enum MlirError {}

#[derive(Clone)]
pub struct MlirCodegenBackend(());

impl MlirCodegenBackend {
    pub fn new() -> Box<dyn CodegenBackend> {
        Box::new(MlirCodegenBackend(()))
    }
}

impl WriteBackendMethods for MlirCodegenBackend {
    type Module = ModuleMlir;
    type TargetMachine = MlirTarget;
    type TargetMachineError = MlirError;
    type ModuleBuffer = ModuleBuffer;
    type ThinData = ThinData;
    type ThinBuffer = ThinBuffer;

    /// Merge all modules into main_module and returning it
    fn run_link(
        cgcx: &CodegenContext<Self>,
        diag_handler: &Handler,
        modules: Vec<ModuleCodegen<Self::Module>>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        todo!();
    }
    /// Performs fat LTO by merging all modules into a single one and returning it
    /// for further optimization.
    fn run_fat_lto(
        cgcx: &CodegenContext<Self>,
        modules: Vec<FatLTOInput<Self>>,
        cached_modules: Vec<(SerializedModule<Self::ModuleBuffer>, WorkProduct)>,
    ) -> Result<LtoModuleCodegen<Self>, FatalError> {
        todo!();
    }
    /// Performs thin LTO by performing necessary global analysis and returning two
    /// lists, one of the modules that need optimization and another for modules that
    /// can simply be copied over from the incr. comp. cache.
    fn run_thin_lto(
        cgcx: &CodegenContext<Self>,
        modules: Vec<(String, Self::ThinBuffer)>,
        cached_modules: Vec<(SerializedModule<Self::ModuleBuffer>, WorkProduct)>,
    ) -> Result<(Vec<LtoModuleCodegen<Self>>, Vec<WorkProduct>), FatalError> {
        todo!();
    }
    fn print_pass_timings(&self) {
        todo!();
    }
    unsafe fn optimize(
        cgcx: &CodegenContext<Self>,
        diag_handler: &Handler,
        module: &ModuleCodegen<Self::Module>,
        config: &ModuleConfig,
    ) -> Result<(), FatalError> {
        todo!();
    }
    fn optimize_fat(
        cgcx: &CodegenContext<Self>,
        llmod: &mut ModuleCodegen<Self::Module>,
    ) -> Result<(), FatalError> {
        todo!();
    }
    unsafe fn optimize_thin(
        cgcx: &CodegenContext<Self>,
        thin: ThinModule<Self>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        todo!();
    }
    unsafe fn codegen(
        cgcx: &CodegenContext<Self>,
        diag_handler: &Handler,
        module: ModuleCodegen<Self::Module>,
        config: &ModuleConfig,
    ) -> Result<CompiledModule, FatalError> {
        todo!();
    }
    fn prepare_thin(module: ModuleCodegen<Self::Module>) -> (String, Self::ThinBuffer) {
        todo!();
    }
    fn serialize_module(module: ModuleCodegen<Self::Module>) -> (String, Self::ModuleBuffer) {
        todo!();
    }
}

impl ExtraBackendMethods for MlirCodegenBackend {
    fn codegen_allocator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        module_name: &str,
        kind: AllocatorKind,
        has_alloc_error_handler: AllocatorKind,
    ) -> ModuleMlir {
        todo!();
    }

    fn compile_codegen_unit(
        &self,
        tcx: TyCtxt<'_>,
        cgu_name: Symbol,
    ) -> (ModuleCodegen<ModuleMlir>, u64) {

        let start_time = Instant::now();

        let dep_node = tcx.codegen_unit(cgu_name).codegen_dep_node(tcx);
        let (module, _) = tcx.dep_graph.with_task(
            dep_node,
            tcx,
            cgu_name,
            module_codegen,
            Some(dep_graph::hash_result),
        );
        let time_to_codegen = start_time.elapsed();

        // We assume that the cost to run LLVM on a CGU is proportional to
        // the time we needed for codegenning it.
        let cost = time_to_codegen.as_nanos() as u64;

        fn module_codegen(tcx: TyCtxt<'_>, cgu_name: Symbol) -> ModuleCodegen<ModuleMlir> {
            let cgu = tcx.codegen_unit(cgu_name);
            let _prof_timer =
                tcx.prof.generic_activity_with_arg_recorder("codegen_module", |recorder| {
                    recorder.record_arg(cgu_name.to_string());
                    recorder.record_arg(cgu.size_estimate().to_string());
                });
            // Instantiate monomorphizations without filling out definitions yet...
            let mlir_module = ModuleMlir::new(tcx, cgu_name.as_str());
            {
                let cx = CodegenCx::new(tcx, cgu, &mlir_module);
                let mono_items = cx.codegen_unit.items_in_deterministic_order(cx.tcx);
                // Predefine functions and globals, so we can refer to them in other
                // definitions.
                for &(mono_item, (linkage, visibility)) in &mono_items {
                    mono_item.predefine::<Builder<'_, '_, '_>>(&cx, linkage, visibility);
                }

                // ... and now that we have everything pre-defined, fill out those definitions.
                for &(mono_item, _) in &mono_items {
                    mono_item.define::<Builder<'_, '_, '_>>(&cx);
                }

                // If this codegen unit contains the main function, also create the
                // wrapper here
                maybe_create_entry_wrapper::<Builder<'_, '_, '_>>(&cx);

                // Finalize code coverage by injecting the coverage map. Note, the coverage map will
                // also be added to the `llvm.compiler.used` variable, created next.
                if cx.sess().instrument_coverage() {
                    cx.coverageinfo_finalize();
                }

                // Finalize debuginfo
                if cx.sess().opts.debuginfo != DebugInfo::None {
                    cx.debuginfo_finalize();
                }
            }
            ModuleCodegen {
                name: cgu_name.to_string(),
                module_llvm: mlir_module,
                kind: ModuleKind::Regular,
            }
        }

        (module, cost)
    }

    fn target_machine_factory(
        &self,
        sess: &Session,
        optlvl: OptLevel,
        target_features: &[String],
    ) -> TargetMachineFactoryFn<Self> {
        // TODO
        Arc::new(|_| Ok(MlirTarget {}))
    }
}

impl CodegenBackend for MlirCodegenBackend {
    fn locale_resource(&self) -> &'static str {
        crate::DEFAULT_LOCALE_RESOURCE
    }

    fn init(&self, sess: &Session) {}

    fn provide(&self, providers: &mut Providers) {
        // TODO compute actual list from cli flags
        providers.global_backend_features = |_tcx, ()| vec![];
    }

    fn print(&self, req: PrintRequest, sess: &Session) {
        todo!();
    }

    fn print_passes(&self) {
        todo!();
    }

    fn print_version(&self) {
        todo!();
    }

    fn target_features(&self, sess: &Session, allow_unstable: bool) -> Vec<Symbol> {
        let v: Vec<Symbol> = vec![];
        v
        // target_features(sess, allow_unstable)
    }

    fn codegen_crate<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        metadata: EncodedMetadata,
        need_metadata_module: bool,
    ) -> Box<dyn Any> {
        Box::new(rustc_codegen_ssa::base::codegen_crate(
            MlirCodegenBackend(()),
            tcx,
            String::from("native"), // TODO replace with actual MLIR target
            metadata,
            need_metadata_module,
        ))
    }

    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn Any>,
        sess: &Session,
        outputs: &OutputFilenames,
    ) -> Result<(CodegenResults, FxHashMap<WorkProductId, WorkProduct>), ErrorGuaranteed> {
        let (codegen_results, work_products) = ongoing_codegen
            .downcast::<rustc_codegen_ssa::back::write::OngoingCodegen<MlirCodegenBackend>>()
            .expect("Expected MlirCodegenBackend's OngoingCodegen, found Box<Any>")
            .join(sess);

        Ok((codegen_results, work_products))
    }

    fn link(
        &self,
        sess: &Session,
        codegen_results: CodegenResults,
        outputs: &OutputFilenames,
    ) -> Result<(), ErrorGuaranteed> {
        // TODO add actual linking whatever that is for MLIR
        Ok(())
    }
}

#[must_use]
pub struct Builder<'a, 'ml, 'tcx> {
    pub cx: &'a CodegenCx<'ml, 'tcx>,
}

impl HasDataLayout for Builder<'_, '_, '_> {
    fn data_layout(&self) -> &TargetDataLayout {
        self.cx.data_layout()
    }
}

impl<'tcx> HasTyCtxt<'tcx> for Builder<'_, '_, 'tcx> {
    #[inline]
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.cx.tcx
    }
}

impl<'tcx> HasParamEnv<'tcx> for Builder<'_, '_, 'tcx> {
    fn param_env(&self) -> ParamEnv<'tcx> {
        self.cx.param_env()
    }
}

impl<'tcx> LayoutOfHelpers<'tcx> for Builder<'_, '_, 'tcx> {
    type LayoutOfResult = TyAndLayout<'tcx>;

    #[inline]
    fn handle_layout_err(&self, err: LayoutError<'tcx>, span: Span, ty: Ty<'tcx>) -> ! {
        self.cx.handle_layout_err(err, span, ty)
    }
}

impl<'tcx> FnAbiOfHelpers<'tcx> for Builder<'_, '_, 'tcx> {
    type FnAbiOfResult = &'tcx FnAbi<'tcx, Ty<'tcx>>;

    #[inline]
    fn handle_fn_abi_err(
        &self,
        err: FnAbiError<'tcx>,
        span: Span,
        fn_abi_request: FnAbiRequest<'tcx>,
    ) -> ! {
        self.cx.handle_fn_abi_err(err, span, fn_abi_request)
    }
}

impl<'ll, 'tcx> Deref for Builder<'_, 'll, 'tcx> {
    type Target = CodegenCx<'ll, 'tcx>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.cx
    }
}

impl<'a, 'ml, 'tcx> HasCodegen<'tcx> for Builder<'a, 'ml, 'tcx> {
    type CodegenCx = CodegenCx<'ml, 'tcx>;
}

impl<'a, 'ml, 'tcx> BackendTypes for Builder<'a, 'ml, 'tcx> {
    type Value = <CodegenCx<'ml, 'tcx> as BackendTypes>::Value;
    type Function = <CodegenCx<'ml, 'tcx> as BackendTypes>::Function;
    type BasicBlock = <CodegenCx<'ml, 'tcx> as BackendTypes>::BasicBlock;
    type Type = <CodegenCx<'ml, 'tcx> as BackendTypes>::Type;
    type Funclet = <CodegenCx<'ml, 'tcx> as BackendTypes>::Funclet;

    type DIScope = <CodegenCx<'ml, 'tcx> as BackendTypes>::DIScope;
    type DILocation = <CodegenCx<'ml, 'tcx> as BackendTypes>::DILocation;
    type DIVariable = <CodegenCx<'ml, 'tcx> as BackendTypes>::DIVariable;
}

impl<'tcx> CoverageInfoBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn set_function_source_hash(
        &mut self,
        instance: Instance<'tcx>,
        function_source_hash: u64,
    ) -> bool {
        todo!();
    }

    fn add_coverage_counter(
        &mut self,
        instance: Instance<'tcx>,
        id: CounterValueReference,
        region: CodeRegion,
    ) -> bool {
        todo!();
    }

    fn add_coverage_counter_expression(
        &mut self,
        instance: Instance<'tcx>,
        id: InjectedExpressionId,
        lhs: ExpressionOperandId,
        op: Op,
        rhs: ExpressionOperandId,
        region: Option<CodeRegion>,
    ) -> bool {
        todo!();
    }

    fn add_coverage_unreachable(&mut self, instance: Instance<'tcx>, region: CodeRegion) -> bool {
        todo!();
    }
}

impl<'ml> DebugInfoBuilderMethods for Builder<'_, 'ml, '_> {
    // FIXME(eddyb) find a common convention for all of the debuginfo-related
    // names (choose between `dbg`, `debug`, `debuginfo`, `debug_info` etc.).
    fn dbg_var_addr(
        &mut self,
        dbg_var: Self::DIVariable,
        dbg_loc: Self::DILocation,
        variable_alloca: Self::Value,
        direct_offset: Size,
        indirect_offsets: &[Size],
        fragment: Option<Range<Size>>,
    ) {
        todo!();
    }

    fn set_dbg_loc(&mut self, dbg_loc: Self::DILocation) {
        todo!();
    }

    fn insert_reference_to_gdb_debug_scripts_section_global(&mut self) {
        todo!();
    }

    fn set_var_name(&mut self, value: Self::Value, name: &str) {
        todo!();
    }
}

impl<'a, 'ml, 'tcx> ArgAbiMethods<'tcx> for Builder<'a, 'ml, 'tcx> {
    fn store_fn_arg(
        &mut self,
        arg_abi: &ArgAbi<'tcx, Ty<'tcx>>,
        idx: &mut usize,
        dst: PlaceRef<'tcx, Self::Value>,
    ) {
        todo!();
    }

    fn store_arg(
        &mut self,
        arg_abi: &ArgAbi<'tcx, Ty<'tcx>>,
        val: Self::Value,
        dst: PlaceRef<'tcx, Self::Value>,
    ) {
        todo!();
    }

    fn arg_memory_ty(&self, arg_abi: &ArgAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        todo!();
    }
}

impl<'a, 'ml, 'tcx> AbiBuilderMethods<'tcx> for Builder<'a, 'ml, 'tcx> {
    fn get_param(&mut self, index: usize) -> Self::Value {
        todo!();
    }
}

impl<'a, 'ml, 'tcx> IntrinsicCallMethods<'tcx> for Builder<'a, 'ml, 'tcx> {
    /// Remember to add all intrinsics here, in `compiler/rustc_hir_analysis/src/check/mod.rs`,
    /// and in `library/core/src/intrinsics.rs`; if you need access to any LLVM intrinsics,
    /// add them to `compiler/rustc_codegen_llvm/src/context.rs`.
    fn codegen_intrinsic_call(
        &mut self,
        instance: Instance<'tcx>,
        fn_abi: &FnAbi<'tcx, Ty<'tcx>>,
        args: &[OperandRef<'tcx, Self::Value>],
        llresult: Self::Value,
        span: Span,
    ) {
        todo!();
    }

    fn abort(&mut self) {
        todo!();
    }
    fn assume(&mut self, val: Self::Value) {
        todo!();
    }
    fn expect(&mut self, cond: Self::Value, expected: bool) -> Self::Value {
        todo!();
    }
    /// Trait method used to test whether a given pointer is associated with a type identifier.
    fn type_test(&mut self, pointer: Self::Value, typeid: Self::Value) -> Self::Value {
        todo!();
    }
    /// Trait method used to load a function while testing if it is associated with a type
    /// identifier.
    fn type_checked_load(
        &mut self,
        llvtable: Self::Value,
        vtable_byte_offset: u64,
        typeid: Self::Value,
    ) -> Self::Value {
        todo!();
    }

    /// Trait method used to inject `va_start` on the "spoofed" `VaListImpl` in
    /// Rust defined C-variadic functions.
    fn va_start(&mut self, val: Self::Value) -> Self::Value {
        todo!();
    }

    /// Trait method used to inject `va_end` on the "spoofed" `VaListImpl` before
    /// Rust defined C-variadic functions return.
    fn va_end(&mut self, val: Self::Value) -> Self::Value {
        todo!();
    }
}

impl<'a, 'ml, 'tcx> AsmBuilderMethods<'tcx> for Builder<'a, 'ml, 'tcx> {
    /// Take an inline assembly expression and splat it out via LLVM
    fn codegen_inline_asm(
        &mut self,
        template: &[InlineAsmTemplatePiece],
        operands: &[InlineAsmOperandRef<'tcx, Self>],
        options: InlineAsmOptions,
        line_spans: &[Span],
        instance: Instance<'_>,
        dest_catch_funclet: Option<(Self::BasicBlock, Self::BasicBlock, Option<&Self::Funclet>)>,
    ) {
        todo!();
    }
}

impl<'a, 'ml, 'tcx> StaticBuilderMethods for Builder<'a, 'ml, 'tcx> {
    fn get_static(&mut self, def_id: DefId) -> Self::Value {
        todo!();
    }
}

impl<'a, 'ml, 'tcx> HasTargetSpec for Builder<'a, 'ml, 'tcx> {
    fn target_spec(&self) -> &Target {
        todo!();
    }
}

impl<'a, 'ml, 'tcx> BuilderMethods<'a, 'tcx> for Builder<'a, 'ml, 'tcx> {
    fn build(cx: &'a Self::CodegenCx, llbb: Self::BasicBlock) -> Self {
        todo!();
    }

    fn cx(&self) -> &CodegenCx<'ml, 'tcx> {
        self.cx
    }

    fn llbb(&self) -> Self::BasicBlock {
        todo!();
    }

    fn set_span(&mut self, _span: Span) {
        todo!();
    }

    fn append_block(
        cx: &'a CodegenCx<'ml, 'tcx>,
        llfn: Self::Function,
        name: &str,
    ) -> Self::BasicBlock {
        todo!();
    }

    fn append_sibling_block(&mut self, name: &str) -> Self::BasicBlock {
        todo!();
    }

    fn switch_to_block(&mut self, llbb: Self::BasicBlock) {
        todo!();
    }

    fn ret_void(&mut self) {
        todo!();
    }

    fn ret(&mut self, v: Self::Value) {
        todo!();
    }

    fn br(&mut self, dest: Self::BasicBlock) {
        todo!();
    }

    fn cond_br(
        &mut self,
        cond: Self::Value,
        then_llbb: Self::BasicBlock,
        else_llbb: Self::BasicBlock,
    ) {
        todo!();
    }

    fn switch(
        &mut self,
        v: Self::Value,
        else_llbb: Self::BasicBlock,
        cases: impl ExactSizeIterator<Item = (u128, Self::BasicBlock)>,
    ) {
        todo!();
    }

    fn invoke(
        &mut self,
        llty: Self::Type,
        fn_abi: Option<&FnAbi<'tcx, Ty<'tcx>>>,
        llfn: Self::Value,
        args: &[Self::Value],
        then: Self::BasicBlock,
        catch: Self::BasicBlock,
        funclet: Option<&Self::Funclet>,
    ) -> Self::Value {
        todo!();
    }

    fn unreachable(&mut self) {
        todo!();
    }

    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn fadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn fadd_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn sub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn fsub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn fsub_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn mul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn fmul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn fmul_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn udiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn exactudiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn sdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn exactsdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn fdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn fdiv_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn urem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn srem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn frem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn frem_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn shl(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn lshr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn ashr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn unchecked_sadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn unchecked_uadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn unchecked_ssub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn unchecked_usub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn unchecked_smul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn unchecked_umul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn and(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn or(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn xor(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }
    fn neg(&mut self, v: Self::Value) -> Self::Value {
        todo!();
    }
    fn fneg(&mut self, v: Self::Value) -> Self::Value {
        todo!();
    }
    fn not(&mut self, v: Self::Value) -> Self::Value {
        todo!();
    }

    fn checked_binop(
        &mut self,
        oop: OverflowOp,
        ty: Ty<'_>,
        lhs: Self::Value,
        rhs: Self::Value,
    ) -> (Self::Value, Self::Value) {
        todo!();
    }

    fn from_immediate(&mut self, val: Self::Value) -> Self::Value {
        todo!();
    }
    fn to_immediate_scalar(&mut self, val: Self::Value, scalar: rustc_target::abi::Scalar) -> Self::Value {
        todo!();
    }

    fn alloca(&mut self, ty: Self::Type, align: Align) -> Self::Value {
        todo!();
    }

    fn byte_array_alloca(&mut self, len: Self::Value, align: Align) -> Self::Value {
        todo!();
    }

    fn load(&mut self, ty: Self::Type, ptr: Self::Value, align: Align) -> Self::Value {
        todo!();
    }

    fn volatile_load(&mut self, ty: Self::Type, ptr: Self::Value) -> Self::Value {
        todo!();
    }

    fn atomic_load(
        &mut self,
        ty: Self::Type,
        ptr: Self::Value,
        order: rustc_codegen_ssa::common::AtomicOrdering,
        size: Size,
    ) -> Self::Value {
        todo!();
    }

    #[instrument(level = "trace", skip(self))]
    fn load_operand(
        &mut self,
        place: PlaceRef<'tcx, Self::Value>,
    ) -> OperandRef<'tcx, Self::Value> {
        todo!();
    }

    fn write_operand_repeatedly(
        &mut self,
        cg_elem: OperandRef<'tcx, Self::Value>,
        count: u64,
        dest: PlaceRef<'tcx, Self::Value>,
    ) {
        todo!();
    }

    fn range_metadata(&mut self, load: Self::Value, range: WrappingRange) {
        todo!();
    }
    fn nonnull_metadata(&mut self, load: Self::Value) {
        todo!();
    }

    fn store(&mut self, val: Self::Value, ptr: Self::Value, align: Align) -> Self::Value {
        todo!();
    }

    fn store_with_flags(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        align: Align,
        flags: MemFlags,
    ) -> Self::Value {
        todo!();
    }

    fn atomic_store(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        order: rustc_codegen_ssa::common::AtomicOrdering,
        size: Size,
    ) {
        todo!();
    }

    fn gep(&mut self, ty: Self::Type, ptr: Self::Value, indices: &[Self::Value]) -> Self::Value {
        todo!();
    }

    fn inbounds_gep(
        &mut self,
        ty: Self::Type,
        ptr: Self::Value,
        indices: &[Self::Value],
    ) -> Self::Value {
        todo!();
    }

    fn struct_gep(&mut self, ty: Self::Type, ptr: Self::Value, idx: u64) -> Self::Value {
        todo!();
    }

    /* Casts */
    fn trunc(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn sext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn fptoui_sat(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn fptosi_sat(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn fptoui(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn fptosi(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn uitofp(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn sitofp(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn fptrunc(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn fpext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn ptrtoint(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn inttoptr(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn bitcast(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn intcast(&mut self, val: Self::Value, dest_ty: Self::Type, is_signed: bool) -> Self::Value {
        todo!();
    }

    fn pointercast(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    /* Comparisons */
    fn icmp(&mut self, op: IntPredicate, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }

    fn fcmp(&mut self, op: RealPredicate, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        todo!();
    }

    /* Miscellaneous instructions */
    fn memcpy(
        &mut self,
        dst: Self::Value,
        dst_align: Align,
        src: Self::Value,
        src_align: Align,
        size: Self::Value,
        flags: MemFlags,
    ) {
        todo!();
    }

    fn memmove(
        &mut self,
        dst: Self::Value,
        dst_align: Align,
        src: Self::Value,
        src_align: Align,
        size: Self::Value,
        flags: MemFlags,
    ) {
        todo!();
    }

    fn memset(
        &mut self,
        ptr: Self::Value,
        fill_byte: Self::Value,
        size: Self::Value,
        align: Align,
        flags: MemFlags,
    ) {
        todo!();
    }

    fn select(
        &mut self,
        cond: Self::Value,
        then_val: Self::Value,
        else_val: Self::Value,
    ) -> Self::Value {
        todo!();
    }

    fn va_arg(&mut self, list: Self::Value, ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn extract_element(&mut self, vec: Self::Value, idx: Self::Value) -> Self::Value {
        todo!();
    }

    fn vector_splat(&mut self, num_elts: usize, elt: Self::Value) -> Self::Value {
        todo!();
    }

    fn extract_value(&mut self, agg_val: Self::Value, idx: u64) -> Self::Value {
        todo!();
    }

    fn insert_value(&mut self, agg_val: Self::Value, elt: Self::Value, idx: u64) -> Self::Value {
        todo!();
    }

    fn set_personality_fn(&mut self, personality: Self::Value) {
        todo!();
    }

    fn cleanup_landing_pad(&mut self, pers_fn: Self::Value) -> (Self::Value, Self::Value) {
        todo!();
    }

    fn resume(&mut self, exn0: Self::Value, exn1: Self::Value) {
        todo!();
    }

    fn cleanup_pad(&mut self, parent: Option<Self::Value>, args: &[Self::Value]) -> Funclet {
        todo!();
    }

    fn cleanup_ret(&mut self, funclet: &Funclet, unwind: Option<Self::BasicBlock>) {
        todo!();
    }

    fn catch_pad(&mut self, parent: Self::Value, args: &[Self::Value]) -> Funclet {
        todo!();
    }

    fn catch_switch(
        &mut self,
        parent: Option<Self::Value>,
        unwind: Option<Self::BasicBlock>,
        handlers: &[Self::BasicBlock],
    ) -> Self::Value {
        todo!();
    }

    // Atomic Operations
    fn atomic_cmpxchg(
        &mut self,
        dst: Self::Value,
        cmp: Self::Value,
        src: Self::Value,
        order: rustc_codegen_ssa::common::AtomicOrdering,
        failure_order: rustc_codegen_ssa::common::AtomicOrdering,
        weak: bool,
    ) -> Self::Value {
        todo!();
    }
    fn atomic_rmw(
        &mut self,
        op: rustc_codegen_ssa::common::AtomicRmwBinOp,
        dst: Self::Value,
        src: Self::Value,
        order: rustc_codegen_ssa::common::AtomicOrdering,
    ) -> Self::Value {
        todo!();
    }

    fn atomic_fence(
        &mut self,
        order: rustc_codegen_ssa::common::AtomicOrdering,
        scope: SynchronizationScope,
    ) {
        todo!();
    }

    fn set_invariant_load(&mut self, load: Self::Value) {
        todo!();
    }

    fn lifetime_start(&mut self, ptr: Self::Value, size: Size) {
        todo!();
    }

    fn lifetime_end(&mut self, ptr: Self::Value, size: Size) {
        todo!();
    }

    fn instrprof_increment(
        &mut self,
        fn_name: Self::Value,
        hash: Self::Value,
        num_counters: Self::Value,
        index: Self::Value,
    ) {
        todo!();
    }

    fn call(
        &mut self,
        llty: Self::Type,
        fn_abi: Option<&FnAbi<'tcx, Ty<'tcx>>>,
        llfn: Self::Value,
        args: &[Self::Value],
        funclet: Option<&Funclet>,
    ) -> Self::Value {
        todo!();
    }

    fn zext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        todo!();
    }

    fn do_not_inline(&mut self, llret: Self::Value) {
        todo!();
    }
}

pub struct ModuleMlir {
    mlcx: mlir_sys::MlirContext,
    mlmod_raw: mlir_sys::MlirModule,
}

impl ModuleMlir {
    fn new(tcx: TyCtxt<'_>, mod_name: &str) -> Self {
        unsafe {
            let mut mlcx = mlir_sys::mlirContextCreate();
            let location = mlir_sys::mlirLocationUnknownGet(mlcx);
            let mlmod_raw = mlir_sys::mlirModuleCreateEmpty(location);
            ModuleMlir { mlcx, mlmod_raw }
        }
    }
}

unsafe impl Send for ModuleMlir {}
unsafe impl Sync for ModuleMlir {}

// impl Drop for ModuleMlir {
    // fn drop(&mut self) {
        // todo!();
    // }
// }
pub struct MlirTarget {}

pub fn target_features(sess: &Session, allow_unstable: bool) -> Vec<Symbol> {
    let mut features: Vec<Symbol> = supported_target_features(sess)
        .iter()
        .filter_map(|&(feature, gate)| {
            if sess.is_nightly_build() || allow_unstable || gate.is_none() {
                Some(feature)
            } else {
                None
            }
        })
        // .filter(|feature| {
        // // check that all features in a given smallvec are enabled
        // for llvm_feature in to_llvm_features(sess, feature) {
        // let cstr = SmallCStr::new(llvm_feature);
        // if !unsafe { llvm::LLVMRustHasFeature(target_machine, cstr.as_ptr()) } {
        // return false;
        // }
        // }
        // true
        // })
        .map(|feature| Symbol::intern(feature))
        .collect();
    features
}
#[no_mangle]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(MlirCodegenBackend(()))
}
