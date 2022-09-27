//! Handles codegen of callees as well as other call-related
//! things. Callees are a superset of normal rust values and sometimes
//! have different representations. In particular, top-level fn items
//! and methods are represented as just a fn ptr and not a full
//! closure.

use crate::CodegenCx;

use mlir_sys;

use rustc_codegen_ssa::traits::*;
use rustc_middle::ty::layout::{FnAbiOf, HasTyCtxt};
use rustc_middle::ty::{self, Instance, TypeVisitableExt};

use std::ffi::CString;

/// Codegens a reference to a fn/method item, monomorphizing and
/// inlining as it goes.
///
/// # Parameters
///
/// - `cx`: the crate context
/// - `instance`: the instance to be instantiated
pub fn get_fn<'ml, 'tcx>(cx: &CodegenCx<'ml, 'tcx>, instance: Instance<'tcx>) -> <CodegenCx<'ml, 'tcx> as BackendTypes>::Function {
    let tcx = cx.tcx();

    debug!("get_fn(instance={:?})", instance);

    assert!(!instance.substs.needs_infer());
    assert!(!instance.substs.has_escaping_bound_vars());

    if let Some(&mlfn) = cx.instances.borrow().get(&instance) {
        return mlfn;
    }

    let sym = tcx.symbol_name(instance).name;
    debug!(
        "get_fn({:?}: {:?}) => {}",
        instance,
        instance.ty(cx.tcx(), ty::ParamEnv::reveal_all()),
        sym
    );

    let fn_abi = cx.fn_abi_of_instance(instance, ty::List::empty());

    unsafe {
        let module_op = mlir_sys::mlirModuleGetOperation(*cx.mlirmod);
        let symtable = mlir_sys::mlirSymbolTableCreate(module_op);
        let sym_strref = mlir_sys::mlirStringRefCreateFromCString(CString::new(sym).unwrap().as_ptr());
        let sym_op = mlir_sys::mlirSymbolTableLookup(symtable, sym_strref);
        let mlfn = if !sym_op.ptr.is_null() {
            sym_op
        } else {
            let instance_def_id = instance.def_id();
            cx.declare_fn(sym, fn_abi)

            // TODO: Set linkage and visibility for the operation.
        };

        cx.instances.borrow_mut().insert(instance, mlfn);

        mlfn
    }
}
