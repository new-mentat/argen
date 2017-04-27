extern crate regex;
extern crate serde_json;

use std::error;
use std::fmt;
use std::io::{Read, Write};
use regex::Regex;

static PERMITTED_C_TYPES: [&'static str; 2] = ["char*", "int"];
static INCLUDES: [&'static str; 4] = ["stdlib", "stdio", "string", "getopt"];

/// c_quote takes a string and quotes it suitably for use in a char* literal in C.
fn c_quote(i: &str) -> String {
    i.replace("\"", "\\\"").replace("\n", "\\n")
}

/// Error type for sanity checks
#[derive(Debug)]
pub struct SanityCheckError {
    descr: String,
}
impl SanityCheckError {
    fn new(s: String) -> SanityCheckError {
        SanityCheckError { descr: s }
    }
}
impl fmt::Display for SanityCheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.descr)
    }
}
impl error::Error for SanityCheckError {
    fn description(&self) -> &str {
        &self.descr
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}


#[derive(Deserialize)]
struct PItem {
    c_var: String,
    c_type: String,
    help_item: String,
    help: Option<String>,
    optional: Option<bool>,
    default: Option<String>,
    multi: Option<bool>, // only valid for last item TODO: actually implement
}

#[derive(Deserialize)]
struct NPItem {
    c_var: String,
    c_type: String,
    long: String,
    short: Option<String>,
    aliases: Option<Vec<String>>,
    help: Option<String>,
    required: Option<bool>,
    no_arg: Option<bool>,
    default: Option<String>,
}

impl NPItem {
    /// declarations for the main function.
    fn decl_main(&self) -> String {
        format!("\t{} {};\n", self.c_type, self.c_var)
    }
    /// declarations for the parse_args (not main) function.
    fn decl_parse(&self) -> String {
        match self.no_arg.unwrap_or(false) {
            true => String::new(),
            false => format!("\tint {}__isset = 0;\n", self.c_var),
        }
    }
    /// assigns value to the c_var in parse loop.
    fn assign(&self) -> String {
        let mut code = String::new();
        match &*self.c_type {
            "int" => match self.no_arg.unwrap_or(false) {
                true  => return format!("\t\t\t*{} = 1;\n", self.c_var),
                false => code.push_str(&format!("\t\t\t*{} = atoi(optarg);\n", self.c_var)),
            },
            "char*" => code.push_str(&format!("\t\t\t*{} = optarg;\n", self.c_var)),
            _ => ()/* impossible (due to sanity check) */,
        }
        match self.no_arg.unwrap_or(false) {
            false => code.push_str(&format!("\t\t\t{}__isset = 1;\n", self.c_var)),
            _ => (),
        }
        code
    }
    /// long option as per getopt_long(3).
    fn long_option(&self, uniq: u8) -> String {
        format!("\t\t{{\"{}\", {}, 0, {}}},\n",
                self.long,
                match self.no_arg.unwrap_or(false) {
                    true => "no_argument",
                    false => "required_argument",
                },
                uniq)
    }
    /// performs checks and conditional assignments after the parse loop.
    fn post_loop(&self) -> String {
        let mut code = String::new();
        code.push_str(&format!("\tif (!{}__isset) {{\n", self.c_var));
        if self.required.unwrap_or(false) {
            code.push_str("\t\tusage(argv[0]);\n");
            code.push_str("\t\texit(1);\n");
        } else if let Some(ref default) = self.default {
            match &*self.c_type {
                "int"   => code.push_str(&format!("\t\t*{} = {};\n", self.c_var, default)),
                "char*" => code.push_str(&format!("\t\t*{} = \"{}\";\n",
                                                  self.c_var,
                                                  c_quote(default))),
                _ => ()/* impossible */,
            }
        } else {
            return String::new();
        }
        code.push_str("\t}\n");
        code
    }
    /// assertion failure when self is invalid.
    fn sanity_check(&self) -> Result<(), SanityCheckError> {
        let identifier_re = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*$").unwrap();
        if !identifier_re.is_match(&self.c_var) {
            return Err(SanityCheckError::new(format!("invalid c variable \"{}\"", self.c_var)));
        }
        let valid_type = (&PERMITTED_C_TYPES)
            .into_iter()
            .any(|&tp| tp == self.c_type);
        if !valid_type {
            return Err(SanityCheckError::new(format!("invalid c type: \"{}\"", self.c_type)));
        }
        if self.long.find(' ').is_some() {
            return Err(SanityCheckError::new(format!("invalid argument long: \"{}\"", self.long)));
        }
        if self.no_arg.unwrap_or(false) {
            if self.c_type != "int" {
                let e = String::from("options that have no_arg set must be of c_type int");
                return Err(SanityCheckError::new(e));
            }
            if self.required.unwrap_or(false) {
                let e = String::from("options that have no_arg set cannot also be required");
                return Err(SanityCheckError::new(e));
            }
        }
        if self.default.is_some() && self.required.unwrap_or(false) {
            let e = String::from("options that are required cannot have a default value");
            return Err(SanityCheckError::new(e));
        }
        if let Some(ref short_name) = self.short {
            if short_name.len() != 1 {
                let e = format!("invalid short name: \"{}\"", short_name);
                return Err(SanityCheckError::new(e));
            }
        }
        if let Some(ref aliases) = self.aliases {
            for alias in aliases {
                if alias.find(' ').is_some() {
                    let e = format!("invalid argument alias: \"{}\"", alias);
                    return Err(SanityCheckError::new(e));
                }
            }
        }
        Ok(())
    }
}


impl PItem {
    /// declarations for the main function.
    fn decl_main(&self) -> String {
        format!("\t{} {};\n", self.c_type, self.c_var)
    }
    /// declarations for the parse function.
    fn decl_parse(&self) -> String {
        match self.optional.unwrap_or(false) {
            true => format!("\tint {}__isset = 0;\n", self.c_var),
            false => String::new(),
        }
    }
    /// assigns value to c_var using argv[0].
    fn assign(&self) -> String {
        let mut code = String::new();
        let tabbing = if self.optional.unwrap_or(false) {
            "\t\t"
        } else {
            "\t"
        };
        match &*self.c_type {
            "int"   => code.push_str(&format!("{}*{} = atoi(argv[0]);\n", tabbing, self.c_var)),
            "char*" => code.push_str(&format!("{}*{} = argv[0];\n", tabbing, self.c_var)),
            _ => ()/* impossible (due to sanity check) */,
        }
        match self.optional.unwrap_or(false) {
            true => code.push_str(&format!("\t\t{}__isset = 1;\n", self.c_var)),
            _ => (),
        }
        code
    }
    fn post_loop(&self) -> String {
        if !self.optional.unwrap_or(false) {
            return String::new();
        }
        let mut code = String::new();
        code.push_str(&format!("\tif (!{}__isset) {{\n", self.c_var));
        if let Some(ref default) = self.default {
            match &*self.c_type {
                "int"   => code.push_str(&format!("\t\t*{} = {};\n", self.c_var, default)),
                "char*" => code.push_str(&format!("\t\t*{} = \"{}\";\n",
                                                  self.c_var,
                                                  c_quote(default))),
                _ => ()/* impossible */,
            }
        } else {
            return String::new();
        }
        code.push_str("\t}\n");
        code
    }
    /// assertion failure when self is invalid.
    fn sanity_check(&self) -> Result<(), SanityCheckError> {
        let identifier_re = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*$").unwrap();
        if !identifier_re.is_match(&self.c_var) {
            return Err(SanityCheckError::new(format!("invalid c variable \"{}\"", self.c_var)));
        }
        let valid_type = (&PERMITTED_C_TYPES)
            .into_iter()
            .any(|&tp| tp == self.c_type);
        if !valid_type {
            return Err(SanityCheckError::new(format!("invalid c type: \"{}\"", self.c_type)));
        }
        if self.default.is_some() && !self.optional.unwrap_or(false) {
            let e = String::from("cannot set default value for non-optional positional argument");
            return Err(SanityCheckError::new(e));
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Spec {
    /// positional must be ordered: required, then optional. only last can be multi.
    positional: Vec<PItem>,
    non_positional: Vec<NPItem>,
}

impl Spec {
    /// deserializes json from a reader into a Spec.
    pub fn from_reader<R>(rdr: R) -> Result<Spec, SanityCheckError>
        where R: Read
    {
        let s: Spec = serde_json::from_reader(rdr).expect("parse json argument spec");
        s.sanity_check()?;
        Ok(s)
    }
    /// check all items in the spec to make sure they are valid.
    fn sanity_check(&self) -> Result<(), SanityCheckError> {
        let mut saw_positional_optional = false;
        for i in 0..self.positional.len() {
            let ref pi = self.positional[i];
            pi.sanity_check()?;
            let o = pi.optional.unwrap_or(false);
            if saw_positional_optional && !o {
                let e = String::from("non-optional positional argument \
                                     cannot come after an optional one");
                return Err(SanityCheckError::new(e));
            }
            if pi.multi.unwrap_or(false) && i != self.positional.len() - 1 {
                let e = String::from("only that last positional argument \
                                     can take multiple values");
                return Err(SanityCheckError::new(e));
            }
            if pi.multi.unwrap_or(false) {
                // TODO: implement and remove this branch
                return Err(SanityCheckError::new(String::from("multi is not yet implemented")));
            }
            if o {
                saw_positional_optional = true
            }
        }
        for npi in &self.non_positional {
            npi.sanity_check()?
        }
        Ok(())
    }
    /// creates the necessary headers in C.
    fn c_headers(&self) -> String {
        INCLUDES
            .iter()
            .map(|s| format!("#include<{}.h>\n", s))
            .collect()
    }
    /// creates the usage function in C.
    fn c_usage(&self) -> String {
        let positional_usage = {
            let mut pos = String::new();
            let mut noptional = 0;
            for pi in &self.positional {
                pos.push(' ');
                if pi.optional.unwrap_or(false) {
                    pos.push('[');
                    noptional += 1;
                }
                pos.push_str(&pi.help_item);
            }
            pos.push_str(&(0..noptional).map(|_| ']').collect::<String>());
            pos
        };
        let mut help = String::new();
        help.push_str(&self.positional
                           .iter()
                           .map(|ref pi| if let Some(ref h) = pi.help {
                                    format!("\t       \"  {}\\n\"\n\t       \"        {}\\n\"\n",
                                            pi.help_item,
                                            &c_quote(&h))
                                } else {
                                    format!("\t       \"  {}\\n\"\n", pi.help_item)
                                })
                           .collect::<String>());
        help.push_str("\t       \"  -h  --help\\n\"\n\
                      \t       \"        print this usage and exit\\n\"\n");
        help.push_str(&self.non_positional
                           .iter()
                           .map(|ref npi| {
            let mut long = String::from("  --");
            long.push_str(&npi.long);
            if let Some(ref aliases) = npi.aliases {
                for alias in aliases {
                    long.push_str("  --");
                    long.push_str(alias);
                }
            }
            let help = match npi.help {
                Some(ref h) => {
                    let mut hm = String::from("\\n\"\n\t       \"        ");
                    hm.push_str(&c_quote(&h));
                    hm
                }
                _ => String::new(),
            };
            if let Some(ref short) = npi.short {
                format!("\t       \"  -{}{}{}\\n\"\n", short, long, help)
            } else {
                format!("\t       \"     {}{}\\n\"\n", long, help)
            }
        })
                           .collect::<String>());
        format!("static void usage(const char *progname) {{\n\
                \tprintf(\"usage: %s [options]{}\\n%s\", progname,\n\
                {}\t       );\n}}\n",
                positional_usage,
                help)
    }
    /// creates the parse_args function in C.
    fn c_parse_args(&self) -> String {
        let mut body = String::new();
        body.push_str("void parse_args(int argc, char **argv");
        for pi in &self.positional {
            body.push_str(&format!(", {} *{}", pi.c_type, pi.c_var))
        }
        for npi in &self.non_positional {
            body.push_str(&format!(", {} *{}", npi.c_type, npi.c_var))
        }
        body.push_str(") {\n");

        // decls, optional
        for npi in &self.non_positional {
            body.push_str(&npi.decl_parse());
        }

        // longopts
        let uniqs: Vec<u8> = self.non_positional
            .iter()
            .map(|npi| {
                     if let Some(ref s) = npi.short {
                         s.as_bytes()[0]
                     } else {
                         1 // TODO: stuff won't work if longopts don't have a shortname
                     }
                 })
            .collect();
        body.push_str("\tstatic struct option longopts[] = {\n");
        for i in 0..self.non_positional.len() {
            let npi = &self.non_positional[i];
            body.push_str(&npi.long_option(uniqs[i]));
        }
        body.push_str("\t\t{\"help\", 0, 0, 'h'},\n");
        body.push_str("\t\t{0, 0, 0, 0}\n\t};\n");

        // shortopts
        let mut optstring = String::from_utf8(self.non_positional
                                                  .iter()
                                                  .filter(|npi| npi.short.is_some())
                                                  .flat_map(|npi| {
            let s = npi.short.clone();
            let mut v = Vec::new();
            v.push(s.unwrap().as_bytes()[0]);
            if !npi.no_arg.unwrap_or(false) {
                v.push(b':');
            }
            v.into_iter().collect::<Vec<u8>>()
        })
                                                  .collect())
                .unwrap();
        optstring.push('h');

        // parse loop, optional
        body.push_str("\tint ch;\n\twhile ((ch = getopt_long(argc, argv, ");
        body.push_str(&format!("\"{}\", longopts, NULL)) != -1) {{\n", optstring));
        body.push_str("\t\tswitch (ch) {\n");
        for i in 0..uniqs.len() {
            body.push_str(&format!("\t\tcase {}:\n{}\t\t\tbreak;\n",
                                   uniqs[i],
                                   self.non_positional[i].assign()));
        }
        body.push_str("\t\tcase 0:\n\t\t\tbreak;\n\
                      \t\tcase 'h':\n\
                      \t\tdefault:\n\t\t\tusage(argv[0]);\n\t\t\texit(1);\n\
                      \t\t}\n\t}\n");

        // post loop, optional
        for npi in &self.non_positional {
            body.push_str(&npi.post_loop());
        }
        body.push_str("\targv += optind;\n\targc -= optind;\n\n");

        // decls, positional
        let required: Vec<&PItem> = self.positional
            .iter()
            .filter(|p| !p.optional.unwrap_or(false))
            .collect();
        let nrequired = required.len();
        for pi in &required {
            body.push_str(&pi.decl_parse());
        }

        // parse loop, positional
        body.push_str(&format!("\tif (argc < {}) {{\n", nrequired));
        body.push_str("\t\tusage(argv[0]);\n\t\texit(1);\n\t}\n");
        for pi in &required {
            body.push_str(&format!("{}\targv++;\n", pi.assign()));
        }
        body.push_str(&format!("\targc -= {};\n\n", nrequired));

        // post loop, positional
        for pi in &required {
            body.push_str(&pi.post_loop());
        }

        // decls, positional optional
        let optional: Vec<&PItem> = self.positional
            .iter()
            .filter(|p| p.optional.unwrap_or(false))
            .collect();
        for pi in &optional {
            body.push_str(&pi.decl_parse());
        }

        // parse loop, positional optional
        for pi in &optional {
            body.push_str("\tif (argc > 0) {\n");
            body.push_str(&pi.assign());
            body.push_str("\t\targv++; argc--;\n\t}\n");
        }

        // post loop, positional optional
        for pi in &optional {
            body.push_str(&pi.post_loop());
        }

        body.push_str("}\n");
        body
    }
    /// creates the main function in C.
    fn c_main(&self) -> String {
        let mut main = String::new();
        main.push_str("int main(int argc, char **argv) {\n");

        for pi in &self.positional {
            main.push_str(&pi.decl_main())
        }
        for npi in &self.non_positional {
            main.push_str(&npi.decl_main())
        }

        main.push_str("\n\tparse_args(argc, argv");
        for pi in &self.positional {
            main.push_str(&format!(", &{}", pi.c_var))
        }
        for npi in &self.non_positional {
            main.push_str(&format!(", &{}", npi.c_var))
        }
        main.push_str(");\n\n");

        main.push_str("\t/* call your code here */\n");
        main.push_str("}\n");
        main
    }
    /// generates everything
    pub fn gen(&self) -> String {
        let h = self.c_headers();
        let usage = self.c_usage();
        let body = self.c_parse_args();
        let main = self.c_main();
        format!("{}\n\n{}\n{}\n{}", h, usage, body, main)
    }
    /// writes generate C code to a writer.
    pub fn writeout<W>(&self, wrt: &mut W)
        where W: Write
    {
        wrt.write_all(self.gen().as_bytes())
            .expect("write generated code to file")
    }
}
