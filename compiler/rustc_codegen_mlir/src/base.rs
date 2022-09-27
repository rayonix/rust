use crate::{Builder, CodegenCx, ModuleMlir};
use rustc_codegen_ssa::mono_item::MonoItemExt;
use rustc_codegen_ssa::traits::*;
use rustc_codegen_ssa::{ModuleCodegen, ModuleKind};
use rustc_middle::dep_graph;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;

use std::time::Instant;

pub fn compile_codegen_unit(tcx: TyCtxt<'_>, cgu_name: Symbol) -> (ModuleCodegen<ModuleMlir>, u64) {
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
        // {
        // let cx = CodegenCx::new(tcx, cgu, &mlir_module);
        // let mono_items = cx.codegen_unit.items_in_deterministic_order(cx.tcx);
        // for &(mono_item, (linkage, visibility)) in &mono_items {
        // mono_item.predefine::<Builder<'_, '_, '_>>(&cx, linkage, visibility);
        // }

        // // ... and now that we have everything pre-defined, fill out those definitions.
        // for &(mono_item, _) in &mono_items {
        // mono_item.define::<Builder<'_, '_, '_>>(&cx);
        // }

        // // If this codegen unit contains the main function, also create the
        // // wrapper here
        // if let Some(entry) = maybe_create_entry_wrapper::<Builder<'_, '_, '_>>(&cx) {
        // let attrs = attributes::sanitize_attrs(&cx, SanitizerSet::empty());
        // attributes::apply_to_llfn(entry, llvm::AttributePlace::Function, &attrs);
        // }

        // // Finalize code coverage by injecting the coverage map. Note, the coverage map will
        // // also be added to the `llvm.compiler.used` variable, created next.
        // if cx.sess().instrument_coverage() {
        // cx.coverageinfo_finalize();
        // }

        // // Create the llvm.used and llvm.compiler.used variables.
        // if !cx.used_statics.borrow().is_empty() {
        // cx.create_used_variable_impl(cstr!("llvm.used"), &*cx.used_statics.borrow());
        // }
        // if !cx.compiler_used_statics.borrow().is_empty() {
        // cx.create_used_variable_impl(
        // cstr!("llvm.compiler.used"),
        // &*cx.compiler_used_statics.borrow(),
        // );
        // }

        // // Run replace-all-uses-with for statics that need it. This must
        // // happen after the llvm.used variables are created.
        // for &(old_g, new_g) in cx.statics_to_rauw().borrow().iter() {
        // unsafe {
        // let bitcast = llvm::LLVMConstPointerCast(new_g, cx.val_ty(old_g));
        // llvm::LLVMReplaceAllUsesWith(old_g, bitcast);
        // llvm::LLVMDeleteGlobal(old_g);
        // }
        // }

        // // Finalize debuginfo
        // if cx.sess().opts.debuginfo != DebugInfo::None {
        // cx.debuginfo_finalize();
        // }
        // }

        ModuleCodegen {
            name: cgu_name.to_string(),
            module_llvm: mlir_module,
            kind: ModuleKind::Regular,
        }
    }

    (module, cost)
}
