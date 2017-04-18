use args::Spec;

pub struct CodeGen {
    spec: Spec,
}

impl CodeGen {
    pub fn new(spec: Spec) -> CodeGen {
        CodeGen { spec: spec }
    }
    /// creates the C function declaration of an argen function,
    /// ending in a `{`.
    pub fn func_decl(&self) -> String {
        String::from("void populate_args(int argc, char ** argv) {")
    }
    /// creates the body of the argen function
    pub fn body(&self) -> String {
        let mut body = String::from("for (int i = 1; i < argc; i++) {");
        // TODO more logic here
        body.push('}');
        body
    }
    /// generates argen.c which features the function argen.
    pub fn gen(&self) -> String {
        let decl = self.func_decl();
        let body = self.body();
        format!("{}{}}}", decl, body)
    }
}
