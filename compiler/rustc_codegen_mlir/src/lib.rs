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

extern crate rustc_codegen_ssa;
extern crate rustc_middle;
extern crate rustc_target;

use melior::Context;
use rustc_codegen_ssa::back::write::CodegenResult;
use rustc_codegen_ssa::traits::*;
use rustc_middle::ty::TyCtxt;
use rustc_target::spec::Target;

mod builder;
mod context;

/// Main structure representing the MLIR backend.
pub struct MlirCodegenBackend;

/// Implement the primary CodegenBackend trait for MLIR.
impl CodegenBackend for MlirCodegenBackend {
    fn codegen_crate(
        &self,
        tcx: TyCtxt<'tcx>,
        _metadata: &[u8],
        _need_metadata_module: bool,
    ) -> CodegenResult<()> {
        Box::new(rustc_codegen_ssa::base::codegen_crate(
            MlirCodegenBackend(()),
            tcx,
            "".to_string(),
            _metadata,
            _need_metadata_module,
        ))
    }

    fn join_codegen(&self, ongoing_codegen: Box<dyn Any>) -> CodegenResult<()> {
        let (codegen_results, work_products) = ongoing_codegen
            .downcast::<rustc_codegen_ssa::back::write::OngoingCodegen<MlirCodegenBackend>>()
            .expect("Expected MlirCodegenBackend's OngoingCodegen, found Box<Any>")
            .join(sess);
        
        // Add any MLIR-specific finalization here if needed
        
        Ok(Box::new((codegen_results, work_products)))
    }

    fn link(
        &self,
        _codegen_results: Vec<CodegenResult<()>>,
        _output: &std::path::Path,
        _linker: &std::path::Path,
    ) -> CodegenResult<()> {
        // TODO: Perform linking or further transformation of MLIR to binary.
        unimplemented!()
    }
    
    fn locale_resource(&self) -> &'static str {
        todo!()
    }
    
    fn init(&self, _sess: &Session) {}
    
    fn print(&self, _req: &PrintRequest, _out: &mut String, _sess: &Session) {}
    
    fn target_features_cfg(&self, _sess: &Session) -> (Vec<Symbol>, Vec<Symbol>) {
        (std::vec![], std::vec![])
    }
    
    fn print_passes(&self) {}
    
    fn print_version(&self) {}
    
    fn metadata_loader(&self) -> Box<MetadataLoaderDyn> {
        Box::new(crate::back::metadata::DefaultMetadataLoader)
    }
    
    fn provide(&self, _providers: &mut Providers) {}
    
    fn supports_parallel(&self) -> bool {
        true
    }
}

impl ExtraBackendMethods for MlirCodegenBackend {
    fn codegen_allocator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        module_name: &str,
        kind: AllocatorKind,
        alloc_error_handler_kind: AllocatorKind,
    ) -> Self::Module {
        todo!()
    }

    fn compile_codegen_unit(
        &self,
        tcx: TyCtxt<'_>,
        cgu_name: Symbol,
    ) -> (ModuleCodegen<Self::Module>, u64) {
        todo!()
    }

    fn target_machine_factory(
        &self,
        sess: &Session,
        opt_level: config::OptLevel,
        target_features: &[String],
    ) -> TargetMachineFactoryFn<Self> {
        todo!()
    }
}

/// Define the backend-specific types.
impl BackendTypes for MlirCodegenBackend {
    type Value = melior::ir::Value;
    type Function = melior::ir::Operation;
    type BasicBlock = melior::ir::Block;
    type Type = melior::ir::Type;
    type Funclet = (); // FIXME: this may be a problem for exception handling

    // Metadata types
    type Metadata = melior::ir::Attribute;

    // Debug info types
    type DIScope = melior::ir::Location;
    type DILocation = melior::ir::Location;
    type DIVariable = melior::ir::Attribute; // Store debug vars as attributes
}
