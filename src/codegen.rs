extern crate regex;
extern crate serde_json;

use std::io::{Read, Write};

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
    }


    /// generate appropriate C code for the particular argument, to be contained within the primary
    /// argument loop. Assume that c_var is an initially-null pointer to a c_type, and
    /// c_var+"__isset" is a boolean. This function should make c_var non-null if applicable, and
    /// if so it sohuld set c_var+"__isset" to true.
    fn gen(&self) -> String {
        let mut code = String::from(format!("\tif(!strcomp(argv[i], {}) && i != argc -1){\n\t{}__isset = true;\n", c_var, c_var));
        match self.c_type{ // TODO; Strings, int arrays, string array 
            "int32" => code.push_str(format!("\t\t*{} = atoi(argv[i+1]);\n\t\t}", c_var)), 
            "char"  => code.push_str(format!("\t\t*{} = argv[i+1][0];\n\t\t})", c_var)), 
        }
        code.push_str(format!("\t\t bool {}__isset == true;\n\t}\n", c_var));
        code.push_str(format!("\t\t arg_count += 1;\n\t}\n", c_var));
        code
    }
    /// generate appropriate C code for after the the primary argument loop. This should check the
    /// c_var+"__isset" value, and if it is false it should either cause the C program to fail with
    /// the help menu or it should assign a default value for c_var. After this is called, if the
    /// program is still running, then c_var MUST be set appropriately.
    fn post_loop(&self) -> String {
        let mut code = String::from(format!("\tif({}__isset = false){\n", c_var));
        if self.default != None {
            match self.c_type {
                "int32" => code.push_str(format!("\t\t*{} = {};\n\t\t}\n}", default)), 
                "char"  => code.push_str(format!("\t\t*{} = {};\n\t\t}\n}", default)), 
            }
        } else {
            if self.required == true {
                // Error, or exit  
            }
        }
        code  
    }
}


impl PItem {
    /// generate approriate variable declarations for this argument, to be contained 
    /// in the main function.
    fn decl(&self) -> String {
    }
    
    fn gen(&self) -> String {
    }

    fn post_loop(&self) -> String{
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
    where R: Read {
        let s = serde_json::from_reader(rdr).expect("parse json argument spec");
        s.sanity_check(); // panic if nonsense input
        s
    }
    /// check all items in the spec to make sure they are valid.
    fn sanity_check(&self) {
        let identifier_re = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*(?:__isset)$").unwrap();
        for pi in &self.positional {
            assert!(identifier_re.is_match(pi.c_var), format!("invalid c variable \"{}\"", pi.c_var));
            let valid_type = (&PERMITTED_C_TYPES).into_iter().any(|tp| tp == pi.c_type);
            assert!(valid_type, format!("invalid c type: \"{}\"", pi.c_type));
        }
        for pi in &self.non_positional {
            assert!(identifier_re.is_match(pi.c_var), format!("invalid c variable \"{}\"", pi.c_var));
            let valid_type = (&PERMITTED_C_TYPES).into_iter().any(|tp| tp == pi.c_type);
            assert!(valid_type, format!("invalid c type: \"{}\"", pi.c_type));
            assert!(pi.name.find(' ').is_none(), "invalid argument name: \"{}\"", pi.name);
            if let Some(short_name) = pi.short {
                assert!(pi.name.len() == 1, "invalid short name: \"{}\"", pi.name);
            }
            if let Some(aliases) = pi.aliases {
                for alias in &aliases {
                    assert!(pi.name.find(' ').is_none(), "invalid argument alias name: \"{}\"", pi.name);
                }
            }
        }
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
            body.push_str(format!("bool {}__isset;\n", npi.c_var));
        }

        // push arg_count variable, which will be used for positional arguments
        body.push_str("int arg_count = 0\n"); 

        // primary loop npitem
        body.push_str("for (int i = 1; i < argc; i++) {");

        // TODO: Add condition for checking whether we have gotten past all positional arguments
        for npi in &self.non_positional {
            body.push_str(npi.gen());
        }
        // post_loop
        body.push('}');

        // primary loop for pitem 
        body.push_str("for (int i = arg_count; i < argc; i++) {");
        for pi in &self.positional {
            body.push_str(pi.gen());
        }

        // post_loop
        body.push('}');
        
        for pi in &self.positional {
            body.push_str(pi.post_loop()); // TODO: Pass relative position index into pi.post_loop
        }

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
        // TODO: ADd Main function 
    }
    /// writes generate C code to a writer.
    pub fn writeout<W>(&self, wrt: W)
    where W: Write {
        w.write_all(self.gen().as_bytes()).expect("write generated code to file")
    }
}
