use rustc_codegen_ssa::traits::BackendTypes;

pub struct CodegenCx {

}

impl BackendTypes for CodegenCx {
    type Value = melior::ir::Value;
    type Metadata = melior::ir::Attribute;
    type Function = melior::ir::Operation;
    type BasicBlock = melior::ir::Block;
    type Type = melior::ir::Type;
    type Funclet = ();
    type DIScope = melior::ir::Location;
    type DILocation = melior::ir::Location;
    type DIVariable = melior::ir::Attribute;
}