use crate::context::CodegenCx;

pub struct Builder<'a, 'll, 'tcx> {
    pub mlircx: &'ll mut mlir::MlirContext<'ll>,
    pub cx: &'a CodegenCx<'ll, 'tcx>,
}

impl<'ll, 'tcx> HasCodegen<'tcx> for Builder<'_, 'll, 'tcx> {
    type CodegenCx = CodegenCx<'ll, 'tcx>;
}

