use crate::CodegenCx;
use mlir_sys;
use rustc_codegen_ssa::traits::*;
use rustc_middle::ty::layout::LayoutOf;
use rustc_middle::ty::Ty;
use rustc_target::abi::call::*;
use rustc_target::abi::AddressSpace;

pub trait MlirType<'ml> {
    fn mlir_type(&self, cx: &CodegenCx<'ml, '_>) -> <CodegenCx<'ml, '_> as BackendTypes>::Type;
    fn ptr_to_mlir_type(&self, cx: &CodegenCx<'ml, '_>) -> <CodegenCx<'ml, '_> as BackendTypes>::Type;
}

pub trait FnAbiMlirExt<'ml, 'tcx> {
    fn mlir_type(&self, cx: &CodegenCx<'ml, 'tcx>) -> <CodegenCx<'ml, 'tcx> as BackendTypes>::Type;
}

impl<'ml, 'tcx> MlirType<'ml> for FnAbi<'tcx, Ty<'tcx>> {
    // Translate the FnAbi into an MLIR type
    fn mlir_type(&self, cx: &CodegenCx<'ml, '_>) -> <CodegenCx<'ml, '_> as BackendTypes>::Type {
        // Ignore "extra" args from the call site for C variadic functions.
        // Only the "fixed" args are part of the LLVM function signature.
        let args =
            if self.c_variadic { &self.args[..self.fixed_count as usize] } else { &self.args };

        // This capacity calculation is approximate.
        let mut llargument_tys = Vec::with_capacity(
            self.args.len() + if let PassMode::Indirect { .. } = self.ret.mode { 1 } else { 0 },
        );

        // Return type
        unsafe {
            let ml_return_ty = match &self.ret.mode {
                PassMode::Ignore => mlir_sys::mlirNoneTypeGet(*cx.mlirctx),
                PassMode::Direct(_) | PassMode::Pair(..) => {
                    todo!()
                    // self.ret.layout.ty.mlir_type(cx)
                },
                PassMode::Cast(cast, _) => {
                    todo!()
                    // cast.mlir_type(cx)
                },
                PassMode::Indirect { .. } => {
                    unimplemented!()
                },
            };

            let type_ = mlir_sys::mlirFunctionTypeGet(*cx.mlirctx, llargument_tys.capacity() as isize, llargument_tys.as_slice().as_ptr(), 1, &ml_return_ty);

            type_
        }
    }

    fn ptr_to_mlir_type(&self, cx: &CodegenCx<'ml, '_>) -> <CodegenCx<'ml, '_> as BackendTypes>::Type {
        unsafe {
            let memory_space = mlir_sys::mlirIntegerAttrGet(mlir_sys::mlirIntegerTypeGet(*cx.mlirctx, 64), AddressSpace::DATA.0 as i64);

            mlir_sys::mlirUnrankedMemRefTypeGet(self.mlir_type(cx), memory_space)
        }
    }

}

