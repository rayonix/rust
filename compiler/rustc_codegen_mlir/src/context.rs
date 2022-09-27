use rustc_middle::ty::{self, Instance, Ty, TyCtxt};

pub struct CodegenCx<'ll, 'tcx> {
    pub tcx: TyCtxt<'tcx>,
}
