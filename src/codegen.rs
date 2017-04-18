extern crate serde_json;

#[derive(Deserialize)]
struct PItem {
    c_var: String,
    c_type: String,
    help: Option<String>,
}

#[derive(Deserialize)]
struct NPItem {
    c_var: String,
    c_type: String,
    name: String,
    short: Option<String>,
    aliases: Option<Vec<String>>,
    help: Option<String>,
    required: Option<bool>,
    default: Option<String>,
}

impl NPItem {
    /// generate appropriate C code for the particular argument, to be contained within the primary
    /// argument loop. Assume that c_var is an initially-null pointer to a c_type, and
    /// c_var+"_isset" is a boolean. This function should make c_var non-null if applicable, and
    /// if so it sohuld set c_var+"_isset" to true.
    fn gen(&self) -> String {
    }
    /// generate appropriate C code for after the the primary argument loop. This should check
    /// the c_var+"_isset" value, and if it is false it should either cause the C program to fail
    /// with the help menu or it should assign a default value for c_var. After this is called, if
    /// the program is still running, then c_var MUST be set appropriately.
    fn post_loop(&self) -> String {
    }
}

#[derive(Deserialize)]
pub struct Spec {
    positional: Vec<PItem>,
    non_positional: Vec<NPItem>,
    c_file: String,
}

impl Spec {
    /// deserializes json from a reader into a Spec.
    pub fn from_reader<R>(rdr: R) -> Spec {
        serde_json::from_reader(rdr).expect("parse json argument spec")
    }
    /// creates the C function declaration of an argen function,
    /// ending in a `{`.
    fn func_decl(&self) -> String {
        String::from("void populate_args(int argc, char ** argv) {")
    }
    /// creates the body of the argen function.
    fn body(&self) -> String {
        let mut body = String::new();

        // create c_var+"_isset" booleans
        for npi in &self.non_positional {
            body.push_str(format!("bool {}_isset;\n", npi.c_var));
        }

        // primary loop
        body.push_str("for (int i = 1; i < argc; i++) {");
        for npi in &self.non_positional {
            body.push_str(npi.gen());
        }

        // post_loop
        body.push('}');
        for npi in &self.non_positional {
            body.push_str(npi.post_loop());
        }

        body
    }
    /// generates argen.c which features the function argen.
    pub fn gen(&self) -> String {
        let decl = self.func_decl();
        let body = self.body();
        format!("{}{}}}", decl, body)
    }
}
