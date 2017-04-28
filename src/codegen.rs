// Argen
// Copyright (C) 2017 Matt Lee <matt@kynelee.com>, Lucas Morales <lucas@lucasem.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

extern crate regex;
extern crate toml;

use std::collections::HashSet;
use std::error;
use std::fmt;
use std::io::Write;
use regex::Regex;

static PERMITTED_C_TYPES: [&str; 2] = ["char*", "int"];
static INCLUDES: [&str; 4] = ["stdlib", "stdio", "string", "getopt"];

/// c_quote takes a string and quotes it suitably for use in a char* literal in C.
fn c_quote(i: &str) -> String {
    i.replace("\"", "\\\"").replace("\n", "\\n")
}

/// Error type for sanity checks
#[derive(Debug)]
pub struct SanityCheckError {
    e: String,
}
impl fmt::Display for SanityCheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.e)
    }
}
impl error::Error for SanityCheckError {
    fn description(&self) -> &str {
        &self.e
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}


#[derive(Deserialize)]
struct PItem {
    c_var: String,
    c_type: String,
    help_name: String,
    help_descr: Option<String>,
    required: Option<bool>,
    default: Option<String>,
    multi: Option<bool>,
    //multi: c_var will be c_type*, and c_var__size will be size_t. default occupies first entry.
    //TODO: multi non-required doesn't compile correctly.
}

#[derive(Deserialize)]
struct NPItem {
    c_var: String,
    c_type: String,
    help_name: Option<String>,
    help_descr: Option<String>,
    long: String,
    aliases: Option<Vec<String>>,
    short: Option<String>,
    required: Option<bool>,
    default: Option<String>,
    flag: Option<bool>,
}

impl NPItem {
    /// declarations for the main function.
    fn decl_main(&self) -> String {
        format!("\t{} {};\n", self.c_type, self.c_var)
    }
    /// declarations for the parse_args (not main) function.
    fn decl_parse(&self) -> String {
        match self.flag.unwrap_or(false) {
            true => String::new(),
            false => format!("\tint {}__isset = 0;\n", self.c_var),
        }
    }
    /// assigns value to the c_var in parse loop.
    fn assign(&self) -> String {
        let mut code = String::new();
        match &*self.c_type {
            "int" => match self.flag.unwrap_or(false) {
                true  => return format!("\t\t\t*{} = 1;\n", self.c_var),
                false => code.push_str(&format!("\t\t\t*{} = atoi(optarg);\n", self.c_var)),
            },
            "char*" => code.push_str(&format!("\t\t\t*{} = optarg;\n", self.c_var)),
            _ => ()/* impossible (due to sanity check) */,
        }
        match self.flag.unwrap_or(false) {
            false => code.push_str(&format!("\t\t\t{}__isset = 1;\n", self.c_var)),
            _ => (),
        }
        code
    }
    /// long option as per getopt_long(3).
    fn long_option(&self, uniq: u8) -> String {
        format!("\t\t{{\"{}\", {}, 0, {}}},\n",
                self.long,
                match self.flag.unwrap_or(false) {
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
            return Err(SanityCheckError { e: format!("invalid c variable \"{}\"", self.c_var) });
        }
        let valid_type = (&PERMITTED_C_TYPES)
            .into_iter()
            .any(|&tp| tp == self.c_type);
        if !valid_type {
            return Err(SanityCheckError { e: format!("invalid c type: \"{}\"", self.c_type) });
        }
        if self.long.find(' ').is_some() {
            return Err(SanityCheckError { e: format!("invalid argument long: \"{}\"", self.long) });
        }
        if self.flag.unwrap_or(false) {
            if self.c_type != "int" {
                let e = String::from("options that are flags must be of c_type int");
                return Err(SanityCheckError { e });
            }
            if self.required.unwrap_or(false) {
                let e = String::from("options that are flags cannot also be required");
                return Err(SanityCheckError { e });
            }
        }
        if self.default.is_some() && self.required.unwrap_or(false) {
            let e = String::from("options that are required cannot have a default value");
            return Err(SanityCheckError { e });
        }
        if let Some(ref short_name) = self.short {
            if short_name.len() != 1 {
                let e = format!("invalid short name: \"{}\"", short_name);
                return Err(SanityCheckError { e });
            }
        }
        if let Some(ref aliases) = self.aliases {
            for alias in aliases {
                if alias.find(' ').is_some() {
                    let e = format!("invalid argument alias: \"{}\"", alias);
                    return Err(SanityCheckError { e });
                }
            }
        }
        Ok(())
    }
}


impl PItem {
    /// declarations for the main function.
    fn decl_main(&self) -> String {
        match self.multi.unwrap_or(false) {
            true => {
                format!("\t{} *{};\n\tsize_t {}__size;\n",
                        self.c_type,
                        self.c_var,
                        self.c_var)
            }
            false => format!("\t{} {};\n", self.c_type, self.c_var),
        }
    }
    /// declarations for the parse function.
    fn decl_parse(&self) -> String {
        if !self.required.unwrap_or(false) && self.default.is_some() {
            format!("\tint {}__isset = 0;\n", self.c_var)
        } else {
            String::new()
        }
    }
    /// assigns value to c_var using argv[0].
    fn assign(&self) -> String {
        let mut code = String::new();
        let tabbing = if self.required.unwrap_or(false) {
            "\t"
        } else {
            "\t\t"
        };
        if self.multi.unwrap_or(false) {
            code.push_str(&format!("{}*{} = argv;\n{}*{}__size = argc;\n",
                                   tabbing,
                                   self.c_var,
                                   tabbing,
                                   self.c_var));
        } else {
            match &*self.c_type {
                "int"   => code.push_str(&format!("{}*{} = atoi(argv[0]);\n", tabbing, self.c_var)),
                "char*" => code.push_str(&format!("{}*{} = argv[0];\n", tabbing, self.c_var)),
                _ => ()/* impossible (due to sanity check) */,
            }
        }
        match !self.required.unwrap_or(false) && self.default.is_some() {
            true => code.push_str(&format!("\t\t{}__isset = 1;\n", self.c_var)),
            _ => (),
        }
        code
    }
    fn post_loop(&self) -> String {
        if self.required.unwrap_or(false) {
            return String::new();
        }
        let mut code = String::new();
        code.push_str(&format!("\tif (!{}__isset) {{\n", self.c_var));
        if let Some(ref default) = self.default {
            if self.multi.unwrap_or(false) {
                code.push_str(&format!("\t\t*{} = &\"{}\\0\";\n\t\t*{}__size = 1\n",
                                       self.c_var,
                                       c_quote(default),
                                       self.c_var))
            } else {
                match &*self.c_type {
                    "int"   => code.push_str(&format!("\t\t*{} = {};\n", self.c_var, default)),
                    "char*" => code.push_str(&format!("\t\t*{} = \"{}\\0\";\n",
                                                      self.c_var,
                                                      c_quote(default))),
                    _ => ()/* impossible */,
                }
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
            return Err(SanityCheckError { e: format!("invalid c variable \"{}\"", self.c_var) });
        }
        let valid_type = (&PERMITTED_C_TYPES)
            .into_iter()
            .any(|&tp| tp == self.c_type);
        if !valid_type {
            return Err(SanityCheckError { e: format!("invalid c type: \"{}\"", self.c_type) });
        }
        if self.required.unwrap_or(false) && self.default.is_some() {
            let e = String::from("cannot set default value for required positional argument");
            return Err(SanityCheckError { e });
        }
        if self.multi.unwrap_or(false) && self.c_type != "char*" {
            let e = String::from("multi-valued argument must be of type char* \
                                 (though they will be stored in char**)");
            return Err(SanityCheckError { e });
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
    /// deserializes toml from a reader into a Spec.
    pub fn from_str(toml: &str) -> Result<Spec, SanityCheckError> {
        let s: Spec = toml::from_str(toml).expect("parse toml argument spec");
        s.sanity_check()?;
        Ok(s)
    }
    /// check all items in the spec to make sure they are valid.
    fn sanity_check(&self) -> Result<(), SanityCheckError> {
        let mut saw_optional = false;
        for i in 0..self.positional.len() {
            let ref pi = self.positional[i];
            pi.sanity_check()?;
            let r = pi.required.unwrap_or(false);
            if saw_optional && r {
                let e = String::from("required positional argument cannot \
                                     come after a non-required one");
                return Err(SanityCheckError { e });
            }
            if pi.multi.unwrap_or(false) && i != self.positional.len() - 1 {
                let e = String::from("only that last positional argument \
                                     can take multiple values");
                return Err(SanityCheckError { e });
            }
            if !r {
                saw_optional = true
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
                if !pi.required.unwrap_or(false) {
                    pos.push('[');
                    noptional += 1;
                }
                pos.push_str(&pi.help_name);
                if pi.multi.unwrap_or(false) {
                    pos.push_str("...");
                }
            }
            pos.push_str(&(0..noptional).map(|_| ']').collect::<String>());
            pos
        };
        let mut help = String::new();
        help.push_str(&self.positional
                           .iter()
                           .map(|ref pi| if let Some(ref d) = pi.help_descr {
                                    format!("\t       \"  {}\\n\"\n\t       \"        {}\\n\"\n",
                                            pi.help_name,
                                            &c_quote(&d))
                                } else {
                                    format!("\t       \"  {}\\n\"\n", pi.help_name)
                                })
                           .collect::<String>());
        help.push_str("\t       \"  -h  --help\\n\"\n\
                      \t       \"        print this usage and exit\\n\"\n");
        help.push_str(&self.non_positional
                           .iter()
                           .map(|ref npi| {
            let mut long = String::from("  --");
            long.push_str(&npi.long);
            if !npi.flag.unwrap_or(false) {
                if let Some(ref help_name) = npi.help_name {
                    long.push_str(&format!(" <{}>", help_name));
                } else {
                    long.push_str(" <arg>")
                }
            }
            if let Some(ref aliases) = npi.aliases {
                long.push_str("  (aliased:");
                for alias in aliases {
                    long.push_str(" --");
                    long.push_str(alias);
                }
                long.push_str(")");
            }
            let descr = match npi.help_descr {
                Some(ref h) => {
                    let mut hm = String::from("\\n\"\n\t       \"        ");
                    hm.push_str(&c_quote(&h));
                    hm
                }
                _ => String::new(),
            };
            if let Some(ref short) = npi.short {
                format!("\t       \"  -{}{}{}\\n\"\n", short, long, descr)
            } else {
                format!("\t       \"    {}{}\\n\"\n", long, descr)
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
            if pi.multi.unwrap_or(false) {
                body.push_str(&format!(", {} **{}, size_t *{}__size",
                                       pi.c_type,
                                       pi.c_var,
                                       pi.c_var))
            } else {
                body.push_str(&format!(", {} *{}", pi.c_type, pi.c_var));
            }
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
        let mut all_bytes: HashSet<u8> = (2..255).collect();
        for npi in &self.non_positional {
            if let Some(ref s) = npi.short {
                all_bytes.remove(&s.as_bytes()[0]);
            }
        }
        let mut iter_bytes = all_bytes.drain();
        let uniqs: Vec<u8> = self.non_positional
            .iter()
            .map(|npi| if let Some(ref s) = npi.short {
                     s.as_bytes()[0]
                 } else {
                     iter_bytes
                         .next()
                         .expect("too many non-positional arguments")
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
            if !npi.flag.unwrap_or(false) {
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

        // decls, positional
        let required: Vec<&PItem> = self.positional
            .iter()
            .filter(|p| p.required.unwrap_or(false) && !p.multi.unwrap_or(false))
            .collect();
        let nrequired =
            required.len() +
            if self.positional
                   .iter()
                   .find(|p| p.required.unwrap_or(false) && p.multi.unwrap_or(false))
                   .is_some() {
                1
            } else {
                0
            };
        for pi in &required {
            body.push_str(&pi.decl_parse());
        }

        // parse loop, positional
        body.push_str(&format!("\n\tif (argc-optind < {}) {{\n", nrequired));
        body.push_str("\t\tusage(argv[0]);\n\t\texit(1);\n\t}\n");
        body.push_str("\targv += optind;\n\targc -= optind;\n\n");
        for pi in &required {
            body.push_str(&format!("{}\targv++;\n", pi.assign()));
        }
        body.push_str(&format!("\targc -= {};\n\n", required.len()));

        // post loop, positional
        for pi in &required {
            body.push_str(&pi.post_loop());
        }

        // decls, positional optional
        let optional: Vec<&PItem> = self.positional
            .iter()
            .filter(|p| !p.required.unwrap_or(false) && !p.multi.unwrap_or(false))
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

        let multi: Option<&PItem> = self.positional
            .iter()
            .find(|p| p.multi.unwrap_or(false));
        if let Some(pi) = multi {
            body.push_str(&pi.decl_parse());
            if pi.required.unwrap_or(false) {
                body.push_str(&pi.assign());
            } else {
                body.push_str("\tif (argc > 0) {\n");
                body.push_str(&pi.assign());
                body.push_str("\t}\n");
            }
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
            main.push_str(&format!(", &{}", pi.c_var));
            if pi.multi.unwrap_or(false) {
                main.push_str(&format!(", &{}__size", pi.c_var))
            }
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
