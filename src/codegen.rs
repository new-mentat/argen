extern crate regex;
extern crate serde_json;

use std::io::{Read, Write};
use regex::Regex;

static PERMITTED_C_TYPES: [&'static str; 2] = ["char*", "int32"]; // TODO: support more types

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
    /// generate approriate variable declarations for this argument, to be contained
    /// in the main function.
    fn decl(&self) -> String {
        String::new()
    }


    /// generate appropriate C code for the particular argument, to be contained within the primary
    /// argument loop. Assume that c_var is an initially-null pointer to a c_type, and
    /// c_var+"__isset" is a boolean. This function should make c_var non-null if applicable, and
    /// if so it sohuld set c_var+"__isset" to true.
    fn gen(&self) -> String {
        let mut code = String::new();
        code.push_str(&format!("\t\tif (!strcmp(argv[i], \"--{}\")) {{\n", self.name));
        match &*self.c_type { // TODO; Strings, int arrays, string array
            "int32" => code.push_str(&format!("\t\t\t*{} = atoi(argv[i+1]);\n", self.c_var)),
            "char*" => code.push_str(&format!("\t\t\t*{} = argv[i+1][0];\n", self.c_var)),
            _ => ()/* impossible (due to sanity check) */,
        }
        code.push_str(&format!("\t\t\t{}__isset = true;\n", self.c_var));
        code.push_str("\t\t\targ_count++;\n");
        code.push_str("\t\t}\n");
        code
    }
    /// generate appropriate C code for after the the primary argument loop. This should check the
    /// c_var+"__isset" value, and if it is false it should either cause the C program to fail with
    /// the help menu or it should assign a default value for c_var. After this is called, if the
    /// program is still running, then c_var MUST be set appropriately.
    fn post_loop(&self) -> String {
        let mut code = String::new();
        code.push_str(&format!("\tif (!{}__isset) {{\n", self.c_var));
        if let Some(ref default) = self.default {
            code.push_str(&format!("\t\t*{} = {};\n", self.c_var, default));
        } else {
            if self.required.unwrap_or(false) {
                // Error, or exit
            }
        }
        code.push_str("\t}\n");
        code
    }
}


impl PItem {
    /// generate approriate variable declarations for this argument, to be contained
    /// in the main function.
    fn decl(&self) -> String {
        String::new()
    }

    fn gen(&self) -> String {
        String::new()
    }

    fn post_loop(&self) -> String {
        String::new()
    }
}

#[derive(Deserialize)]
pub struct Spec {
    positional: Vec<PItem>,
    non_positional: Vec<NPItem>,
}

impl Spec {
    /// deserializes json from a reader into a Spec.
    pub fn from_reader<R>(rdr: R) -> Spec
        where R: Read
    {
        let s: Spec = serde_json::from_reader(rdr).expect("parse json argument spec");
        s.sanity_check(); // panic if nonsense input
        s
    }
    /// check all items in the spec to make sure they are valid.
    fn sanity_check(&self) {
        let identifier_re = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*$").unwrap();
        for pi in &self.positional {
            assert!(identifier_re.is_match(&pi.c_var),
                    format!("invalid c variable \"{}\"", pi.c_var));
            let valid_type = (&PERMITTED_C_TYPES)
                .into_iter()
                .any(|&tp| tp == pi.c_type);
            assert!(valid_type, format!("invalid c type: \"{}\"", pi.c_type));
        }
        for pi in &self.non_positional {
            assert!(identifier_re.is_match(&pi.c_var),
                    format!("invalid c variable \"{}\"", pi.c_var));
            let valid_type = (&PERMITTED_C_TYPES)
                .into_iter()
                .any(|&tp| tp == pi.c_type);
            assert!(valid_type, format!("invalid c type: \"{}\"", pi.c_type));
            assert!(pi.name.find(' ').is_none(),
                    "invalid argument name: \"{}\"",
                    pi.name);
            if let Some(ref short_name) = pi.short {
                assert!(short_name.len() == 1,
                        "invalid short name: \"{}\"",
                        short_name);
            }
            if let Some(ref aliases) = pi.aliases {
                for alias in aliases {
                    assert!(alias.find(' ').is_none(),
                            "invalid argument alias name: \"{}\"",
                            alias);
                }
            }
        }
    }
    /// creates the C function declaration of an argen function,
    /// ending in a `{`.
    fn func_decl(&self) -> String {
        String::from("void populate_args(int argc, char ** argv /* TODO */) {\n")

    }
    /// creates the body of the argen function.
    fn body(&self) -> String {
        let mut body = String::new();

        // create c_var+"_isset" booleans
        for npi in &self.non_positional {
            body.push_str(&format!("\tbool {}__isset = false;\n", npi.c_var));
        }

        // push arg_count variable, which will be used for positional arguments
        body.push_str("\tint arg_count = 0\n");

        // primary loop npitem
        body.push_str("\tfor (int i = 1; i < argc; i++) {\n");

        // TODO: Add condition for checking whether we have gotten past all positional arguments
        for npi in &self.non_positional {
            body.push_str(&npi.gen());
        }
        body.push_str("\t}\n");

        // primary loop for pitem
        body.push_str("\tfor (int i = arg_count; i < argc; i++) {\n");
        for pi in &self.positional {
            body.push_str(&pi.gen());
        }
        body.push_str("\t}\n");

        // post_loop
        for pi in &self.positional {
            body.push_str(&pi.post_loop()); // TODO: Pass relative position index into pi.post_loop
        }

        for npi in &self.non_positional {
            body.push_str(&npi.post_loop());
        }

        body
    }
    /// generates argen.c which features the function argen.
    pub fn gen(&self) -> String {
        let decl = self.func_decl();
        let body = self.body();
        format!("{}{}}}", decl, body)
        // TODO: Add Main function
    }
    /// writes generate C code to a writer.
    pub fn writeout<W>(&self, wrt: &mut W)
        where W: Write
    {
        wrt.write_all(self.gen().as_bytes())
            .expect("write generated code to file")
    }
}
