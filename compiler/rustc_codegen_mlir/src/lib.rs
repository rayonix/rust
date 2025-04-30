#![allow(unused)]
#![feature(rustc_private)]
//! # MLIR Codegen Backend
//!
//! This file sets up the trait definitions and empty functions required
//! by `rustc_codegen_ssa` for a new MLIR backend.
//!
//! References:
//! - Rust compiler codegen: https://github.com/rust-lang/rust/tree/master/compiler/rustc_codegen_ssa
//! - MLIR: https://mlir.llvm.org

extern crate rustc_ast;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

use std::any::Any;
use std::hash::Hash;
use std::path::Path;

use melior::Context;
use melior::utility::register_all_dialects;
use rustc_ast::expand::allocator::AllocatorKind;
use rustc_ast::expand::autodiff_attrs::AutoDiffItem;
use rustc_codegen_ssa::back::lto::{self, LtoModuleCodegen, SerializedModule, ThinModule};
use rustc_codegen_ssa::back::write::{
    CodegenContext, FatLtoInput, ModuleConfig, OngoingCodegen, TargetMachineFactoryConfig,
    TargetMachineFactoryFn,
};
use rustc_codegen_ssa::traits::*;
use rustc_codegen_ssa::{
    CodegenResults, CompiledModule, CrateInfo, MemFlags, ModuleCodegen, ModuleKind,
};
use rustc_data_structures::fx::FxIndexMap;
use rustc_errors::{DiagCtxtHandle, ErrorGuaranteed, FatalError};
use rustc_metadata::EncodedMetadata;
use rustc_metadata::creader::MetadataLoaderDyn;
use rustc_middle::dep_graph::{WorkProduct, WorkProductId};
use rustc_middle::ty::TyCtxt;
use rustc_middle::util::Providers;
use rustc_session::config::{OutputFilenames, PrintRequest};
use rustc_session::{Session, config};
use rustc_span::Symbol;
use rustc_target::spec::Target;

mod builder;
mod context;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Dummy;

/// Buffer type implementing ModuleBufferMethods and ThinBufferMethods
pub struct Buffer;

impl ModuleBufferMethods for Buffer {
    fn data(&self) -> &[u8] {
        &[]
    }
}

impl ThinBufferMethods for Buffer {
    fn data(&self) -> &[u8] {
        &[]
    }
    fn thin_link_data(&self) -> &[u8] {
        &[]
    }
}

/// Main structure representing the MLIR backend.
#[derive(Clone, Copy)]
pub struct MlirCodegenBackend;

/// Implement the primary CodegenBackend trait for MLIR.
impl CodegenBackend for MlirCodegenBackend {
    fn locale_resource(&self) -> &'static str {
        todo!()
    }

    fn init(&self, _sess: &Session) {
        let registry = melior::dialect::DialectRegistry::new();
        register_all_dialects(&registry);
        let context = melior::Context::new();
        context.append_dialect_registry(&registry);
        context.load_all_available_dialects();
    }

    fn print(&self, _req: &PrintRequest, _out: &mut String, _sess: &Session) {}

    fn target_features_cfg(&self, _sess: &Session) -> (Vec<Symbol>, Vec<Symbol>) {
        (std::vec![], std::vec![])
    }

    fn print_passes(&self) {}

    fn print_version(&self) {}

    fn provide(&self, _providers: &mut Providers) {}

    fn codegen_crate<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        metadata: rustc_metadata::EncodedMetadata,
        _need_metadata_module: bool,
    ) -> Box<dyn Any> {
        Box::new(rustc_codegen_ssa::base::codegen_crate(
            MlirCodegenBackend,
            tcx,
            "".to_string(),
            metadata,
            _need_metadata_module,
        ))
    }

    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn Any>,
        sess: &Session,
        outputs: &OutputFilenames,
    ) -> (CodegenResults, FxIndexMap<WorkProductId, WorkProduct>) {
        ongoing_codegen
            .downcast::<OngoingCodegen<MlirCodegenBackend>>()
            .expect("Expected MlirCodegenBackend's OngoingCodegen, found Box<Any>")
            .join(sess)
    }

    fn link(&self, sess: &Session, codegen_results: CodegenResults, outputs: &OutputFilenames) {
        rustc_codegen_ssa::back::link::link_binary(
            sess,
            &rustc_codegen_ssa::back::archive::ArArchiveBuilderBuilder,
            codegen_results,
            outputs,
        );
    }

    fn supports_parallel(&self) -> bool {
        true
    }
}

impl WriteBackendMethods for MlirCodegenBackend {
    type Module = ();
    type TargetMachine = ();
    type ModuleBuffer = Buffer;
    type ThinData = Buffer;
    type ThinBuffer = Buffer;
    type TargetMachineError = String;

    fn run_link(
        _cgcx: &CodegenContext<Self>,
        _dcx: DiagCtxtHandle<'_>,
        _modules: Vec<ModuleCodegen<Self::Module>>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        todo!()
    }

    fn run_fat_lto(
        _cgcx: &CodegenContext<Self>,
        _modules: Vec<FatLtoInput<Self>>,
        _cached_modules: Vec<(SerializedModule<Self::ModuleBuffer>, WorkProduct)>,
    ) -> Result<LtoModuleCodegen<Self>, FatalError> {
        todo!()
    }

    fn run_thin_lto(
        _cgcx: &CodegenContext<Self>,
        _modules: Vec<(String, Self::ThinBuffer)>,
        _cached_modules: Vec<(SerializedModule<Self::ModuleBuffer>, WorkProduct)>,
    ) -> Result<(Vec<LtoModuleCodegen<Self>>, Vec<WorkProduct>), FatalError> {
        todo!()
    }

    fn print_pass_timings(&self) {
        todo!()
    }

    fn print_statistics(&self) {
        todo!()
    }

    unsafe fn optimize(
        _cgcx: &CodegenContext<Self>,
        _dcx: DiagCtxtHandle<'_>,
        _module: &mut ModuleCodegen<Self::Module>,
        _config: &ModuleConfig,
    ) -> Result<(), FatalError> {
        todo!()
    }

    fn optimize_fat(
        _cgcx: &CodegenContext<Self>,
        _llmod: &mut ModuleCodegen<Self::Module>,
    ) -> Result<(), FatalError> {
        todo!()
    }

    unsafe fn optimize_thin(
        _cgcx: &CodegenContext<Self>,
        _thin: ThinModule<Self>,
    ) -> Result<ModuleCodegen<Self::Module>, FatalError> {
        todo!()
    }

    unsafe fn codegen(
        _cgcx: &CodegenContext<Self>,
        _dcx: DiagCtxtHandle<'_>,
        _module: ModuleCodegen<Self::Module>,
        _config: &ModuleConfig,
    ) -> Result<CompiledModule, FatalError> {
        todo!()
    }

    fn prepare_thin(
        _module: ModuleCodegen<Self::Module>,
        _want_summary: bool,
    ) -> (String, Self::ThinBuffer) {
        todo!()
    }

    fn serialize_module(_module: ModuleCodegen<Self::Module>) -> (String, Self::ModuleBuffer) {
        todo!()
    }

    fn autodiff(
        _cgcx: &CodegenContext<Self>,
        _module: &ModuleCodegen<Self::Module>,
        _diff_fncs: Vec<AutoDiffItem>,
        _config: &ModuleConfig,
    ) -> Result<(), FatalError> {
        todo!()
    }
}

impl ExtraBackendMethods for MlirCodegenBackend {
    fn codegen_allocator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        _module_name: &str,
        _kind: AllocatorKind,
        _alloc_error_handler_kind: AllocatorKind,
    ) -> Self::Module {
        todo!()
    }

    fn compile_codegen_unit(
        &self,
        tcx: TyCtxt<'_>,
        cgu_name: Symbol,
    ) -> (rustc_codegen_ssa::ModuleCodegen<Self::Module>, u64) {
        todo!()
    }

    fn target_machine_factory(
        &self,
        sess: &Session,
        opt_level: config::OptLevel,
        _target_features: &[String],
    ) -> TargetMachineFactoryFn<Self> {
        use std::sync::Arc;
        Arc::new(|_| Err("Target machine creation not implemented".to_string()))
    }
}

// /// Define the backend-specific types.
// impl BackendTypes for MlirCodegenBackend {
//     type Value = melior::ir::Value<'static, 'static>;
//     type Function = melior::ir::Operation<'static>;
//     type BasicBlock = melior::ir::Block<'static>;
//     type Type = melior::ir::Type<'static, 'static>;
//     type Funclet = ();

//     type Metadata = melior::ir::Attribute<'static>;

//     type DIScope = melior::ir::Location<'static>;
//     type DILocation = melior::ir::Location<'static>;
//     type DIVariable = melior::ir::Attribute<'static>;
// }
