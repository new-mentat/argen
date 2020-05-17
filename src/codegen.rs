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

use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io::Write;

const INCLUDES: [&str; 4] = ["stdlib", "stdio", "string", "getopt"];

const HELP_PREFIX: &str = "\t       \"  ";

/// c_quote takes a string and quotes it suitably for use in a char* literal in C.
fn c_quote(i: &str) -> String {
    i.replace("\"", "\\\"").replace("\n", "\\n")
}

/// Error type for sanity checks
#[derive(Debug)]
pub enum ValidationError {
    TomlError(toml::de::Error),
    BadIdent(String, String),
    RequiredHasDefault(String),
    MultiNotChars(String),
    InvalidLong(String),
    InvalidShort(String, String),
    InvalidAlias(String, String),
    FlagMustBeInt(String),
    FlagHasDefault(String),
    FlagCannotBeRequired(String),
    RequiredPositionalGoesBeforeOptionPositional(String),
    MultiMustBeLast(String),
}
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::TomlError(e) => e.fmt(f),
            ValidationError::BadIdent(param, ident) =>
                write!(f, "in param {}: invalid c variable \"{}\"", param, ident),
            ValidationError::RequiredHasDefault(param) =>
                write!(f, "in param {}: cannot set default value for required argument", param),
            ValidationError::MultiNotChars(param) =>
                write!(f, "in param {}: multi-valued argument must be of type char* (though they will be stored in char**)", param),
            ValidationError::InvalidLong(long) =>
                write!(f, "invalid argument long: \"{}\"", long),
            ValidationError::InvalidShort(param, short) =>
                write!(f, "in param {}: invalid short name: \"{}\"", param, short),
            ValidationError::InvalidAlias(param, alias) =>
                write!(f, "in param {}: invalid argument alias: \"{}\"", param, alias),
            ValidationError::FlagMustBeInt(param) =>
                write!(f, "in param {}: options that are flags must be of c_type int", param),
            ValidationError::FlagHasDefault(param) =>
                write!(f, "in param {}: options that are flags cannot have default", param),
            ValidationError::FlagCannotBeRequired(param) =>
                write!(f, "in param {}: options that are flags cannot also be required", param),
            ValidationError::RequiredPositionalGoesBeforeOptionPositional(param) =>
                write!(f, "in param {}: required positional argument cannot come after a non-required one", param),
            ValidationError::MultiMustBeLast(param) =>
                write!(f, "in param {}: only the last positional argument can take multiple values", param),
        }
    }
}
impl Error for ValidationError {}
impl From<toml::de::Error> for ValidationError {
    fn from(err: toml::de::Error) -> ValidationError {
        ValidationError::TomlError(err)
    }
}

#[derive(Clone, Copy, Deserialize)]
enum CType {
    #[serde(rename = "char*")]
    Chars,
    #[serde(rename = "int")]
    Int,
}
impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CType::Chars => write!(f, "char*"),
            CType::Int => write!(f, "int"),
        }
    }
}

#[derive(Deserialize)]
struct PositionalItem {
    c_var: String,
    c_type: CType,
    help_name: String,
    help_descr: Option<String>,
    required: Option<bool>,
    default: Option<String>,
    //multi: c_var will be c_type*, and c_var__size will be size_t. default occupies first entry.
    multi: Option<bool>,
}

impl PositionalItem {
    fn is_required(&self) -> bool {
        self.required.unwrap_or(false)
    }
    fn is_multi(&self) -> bool {
        self.multi.unwrap_or(false)
    }
    fn has_default(&self) -> bool {
        self.default.is_some()
    }
    /// A suitable string to go into the parse_args declaration. Starts with ',' if anything.
    fn cgen_decl_arg(&self) -> String {
        if self.is_multi() {
            format!(", {} **{}, size_t *{1}__size", self.c_type, self.c_var)
        } else {
            format!(", {} *{}", self.c_type, self.c_var)
        }
    }
    /// A suitable string to go into the parse_args function call. Starts with ',' if anything.
    fn cgen_call_arg(&self) -> String {
        if self.is_multi() {
            format!(", &{}, &{0}__size", self.c_var)
        } else {
            format!(", &{}", self.c_var)
        }
    }
    /// Declarations for the main function.
    fn cgen_main_decls(&self) -> String {
        if self.is_multi() {
            format!("\t{} *{};\n\tsize_t {1}__size;\n", self.c_type, self.c_var)
        } else {
            format!("\t{} {};\n", self.c_type, self.c_var)
        }
    }
    /// Declaration of __isset variables for the parse_args (not main) function.
    fn cgen_isset_decl(&self) -> String {
        if self.has_default() {
            format!("\tint {}__isset = 0;\n", self.c_var)
        } else {
            String::new()
        }
    }
    /// Definition of __default variables for the parse_args (not main) function.
    fn cgen_default_decl(&self) -> String {
        match &self.default {
            Some(default) => {
                let quoted = format!("\"{}\"", c_quote(default));
                let default = match self.c_type {
                    CType::Chars => &quoted,
                    CType::Int => default,
                };
                format!(
                    "\tstatic {} {}__default = {};\n",
                    self.c_type, self.c_var, default
                )
            }
            _ => String::new(),
        }
    }
    /// Assigns value to c_var using argv[0].
    fn cgen_assign_argv0(&self) -> String {
        let indent = if self.is_required() { "\t" } else { "\t\t" };
        let set_isset = if self.has_default() {
            format!("{}{}__isset = 1;\n", indent, self.c_var)
        } else {
            String::new()
        };
        if self.is_multi() {
            format!(
                "{}*{} = argv;\n{0}*{1}__size = argc;\n{}",
                indent, self.c_var, set_isset
            )
        } else {
            match self.c_type {
                CType::Chars => format!("{}*{} = argv[0];\n{}", indent, self.c_var, set_isset),
                CType::Int => format!("{}*{} = atoi(argv[0]);\n{}", indent, self.c_var, set_isset),
            }
        }
    }
    /// Performs checks and conditional assignments after the parse loop.
    fn cgen_post_loop(&self) -> String {
        if self.has_default() {
            let if_blk = format!("\tif (!{}__isset) {{\n", self.c_var);
            if self.is_multi() {
                format!(
                    "{}\t\t*{} = &{1}__default;\n\t\t*{1}__size = 1;\n\t}}\n",
                    if_blk, self.c_var
                )
            } else {
                format!("{}\t\t*{} = {1}__default;\n\t}}\n", if_blk, self.c_var)
            }
        } else {
            String::new()
        }
    }
    /// Error if self is invalid.
    fn validate(&self) -> Result<(), ValidationError> {
        let identifier_re = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*$").unwrap();
        if !identifier_re.is_match(&self.c_var) {
            return Err(ValidationError::BadIdent(
                self.help_name.to_owned(),
                self.c_var.to_owned(),
            ));
        }
        if self.is_required() && self.has_default() {
            return Err(ValidationError::RequiredHasDefault(
                self.help_name.to_owned(),
            ));
        }
        if self.is_multi() {
            if let CType::Int = self.c_type {
                return Err(ValidationError::MultiNotChars(self.help_name.to_owned()));
            }
        }
        Ok(())
    }
    fn help(&self) -> String {
        if let Some(d) = &self.help_descr {
            format!(
                "{}{}\\n\"\n{0}      {}\\n\"\n",
                HELP_PREFIX,
                self.help_name,
                &c_quote(&d)
            )
        } else {
            format!("{}{}\\n\"\n", HELP_PREFIX, self.help_name)
        }
    }
}

#[derive(Deserialize)]
struct NonPositionalItem {
    c_var: String,
    c_type: CType,
    long: String,
    help_name: Option<String>,
    help_descr: Option<String>,
    aliases: Option<Vec<String>>,
    short: Option<String>,
    required: Option<bool>,
    default: Option<String>,
    flag: Option<bool>,
}

impl NonPositionalItem {
    fn is_flag(&self) -> bool {
        self.flag.unwrap_or(false)
    }
    fn is_required(&self) -> bool {
        self.required.unwrap_or(false)
    }
    fn has_default(&self) -> bool {
        self.default.is_some()
    }
    /// A suitable string to go into the parse_args declaration. Starts with ',' if anything.
    fn cgen_decl_arg(&self) -> String {
        format!(", {} *{}", self.c_type, self.c_var)
    }
    /// A suitable string to go into the parse_args function call. Starts with ',' if anything.
    fn cgen_call_arg(&self) -> String {
        format!(", &{}", self.c_var)
    }
    /// Declarations for the main function.
    fn cgen_main_decl(&self) -> String {
        format!("\t{} {};\n", self.c_type, self.c_var)
    }
    /// Declaration of __isset variables for the parse_args (not main) function.
    fn cgen_isset_decl(&self) -> String {
        if !self.is_flag() {
            format!("\tint {}__isset = 0;\n", self.c_var)
        } else {
            String::new()
        }
    }
    /// Definition of __default variables for the parse_args (not main) function.
    fn cgen_default_decl(&self) -> String {
        match &self.default {
            Some(default) => {
                let quoted = format!("\"{}\"", c_quote(default));
                let default = match self.c_type {
                    CType::Chars => &quoted,
                    CType::Int => default,
                };
                format!(
                    "\tstatic {} {}__default = {};\n",
                    self.c_type, self.c_var, default
                )
            }
            _ => String::new(),
        }
    }
    /// Assigns value to the c_var in parse loop.
    fn cgen_assign_optarg(&self) -> String {
        if self.is_flag() {
            format!("\t\t\t*{} = 1;\n", self.c_var)
        } else {
            let set_isset = format!("\t\t\t{}__isset = 1;\n", self.c_var);
            match self.c_type {
                CType::Chars => format!("\t\t\t*{} = optarg;\n{}", self.c_var, set_isset),
                CType::Int => format!("\t\t\t*{} = atoi(optarg);\n{}", self.c_var, set_isset),
            }
        }
    }
    /// Long option as per getopt_long(3).
    fn cgen_getopt(&self, uniq: u8) -> String {
        format!(
            "\t\t{{\"{}\", {}, 0, {}}},\n",
            self.long,
            if self.is_flag() {
                "no_argument"
            } else {
                "required_argument"
            },
            uniq
        )
    }
    /// Performs checks and conditional assignments after the parse loop.
    fn cgen_post_loop(&self) -> String {
        if self.is_required() {
            format!(
                "\tif (!{}__isset) {{\n\t\tusage(argv[0]);\n\t\texit(1);\n\t}}\n",
                self.c_var
            )
        } else if self.default.is_none() {
            String::new()
        } else {
            format!(
                "\tif (!{}__isset) {{\n\t\t*{0} = {0}__default;\n\t}}\n",
                self.c_var
            )
        }
    }
    /// Error if self is invalid.
    fn validate(&self) -> Result<(), ValidationError> {
        let identifier_re = Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*$").unwrap();
        if !identifier_re.is_match(&self.c_var) {
            return Err(ValidationError::BadIdent(
                self.long.to_owned(),
                self.c_var.to_owned(),
            ));
        }
        if self.long.find(' ').is_some() {
            return Err(ValidationError::InvalidLong(self.long.to_owned()));
        }
        if self.is_flag() {
            if let CType::Chars = self.c_type {
                return Err(ValidationError::FlagMustBeInt(self.long.to_owned()));
            }
            if self.has_default() {
                return Err(ValidationError::FlagHasDefault(self.long.to_owned()));
            }
            if self.is_required() {
                return Err(ValidationError::FlagCannotBeRequired(self.long.to_owned()));
            }
        }
        if self.has_default() && self.is_required() {
            return Err(ValidationError::RequiredHasDefault(self.long.to_owned()));
        }
        if let Some(short_name) = &self.short {
            if short_name.len() != 1 {
                return Err(ValidationError::InvalidShort(
                    self.long.to_owned(),
                    short_name.to_owned(),
                ));
            }
        }
        if let Some(aliases) = &self.aliases {
            for alias in aliases {
                if alias.find(' ').is_some() {
                    return Err(ValidationError::InvalidAlias(
                        self.long.to_owned(),
                        alias.to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }
    fn help(&self) -> String {
        let mut long = String::from("  --");
        long.push_str(&self.long);
        if !self.is_flag() {
            if let Some(help_name) = &self.help_name {
                long.push_str(&format!(" <{}>", help_name));
            } else {
                long.push_str(" <arg>")
            }
        }
        if let Some(aliases) = &self.aliases {
            long.push_str("  (aliased:");
            for alias in aliases {
                long.push_str(" --");
                long.push_str(alias);
            }
            long.push_str(")");
        }
        let descr = match &self.help_descr {
            Some(h) => {
                let mut hm = String::from("\\n\"\n\t       \"        ");
                hm.push_str(&c_quote(&h));
                hm
            }
            _ => String::new(),
        };
        if let Some(short) = &self.short {
            format!("{}-{}{}{}\\n\"\n", HELP_PREFIX, short, long, descr)
        } else {
            format!("{}  {}{}\\n\"\n", HELP_PREFIX, long, descr)
        }
    }
}

#[derive(Deserialize)]
pub struct Spec {
    /// Positional must be ordered: required, then optional.
    /// Only the last PositionalItem can be multi.
    positional: Vec<PositionalItem>,
    /// Non-positional is unordered.
    non_positional: Vec<NonPositionalItem>,
}

impl Spec {
    /// Deserializes toml from a string into a Spec.
    pub fn from_str(toml: &str) -> Result<Spec, ValidationError> {
        let s: Spec = toml::from_str(toml)?;
        s.validate()?;
        Ok(s)
    }
    /// Check all items in the spec to make sure they are valid.
    fn validate(&self) -> Result<(), ValidationError> {
        let mut saw_optional = false;
        for (i, pi) in self.positional.iter().enumerate() {
            pi.validate()?;
            if pi.is_required() && saw_optional {
                return Err(
                    ValidationError::RequiredPositionalGoesBeforeOptionPositional(
                        pi.help_name.to_owned(),
                    ),
                );
            }
            if pi.is_multi() && i != self.positional.len() - 1 {
                return Err(ValidationError::MultiMustBeLast(pi.help_name.to_owned()));
            }
            if !pi.is_required() {
                saw_optional = true
            }
        }
        for npi in &self.non_positional {
            npi.validate()?
        }
        Ok(())
    }
    /// Creates the necessary headers in C.
    fn cgen_headers(&self) -> String {
        INCLUDES
            .iter()
            .map(|s| format!("#include<{}.h>\n", s))
            .collect()
    }
    /// Creates the usage function in C.
    fn cgen_usage(&self) -> String {
        let positional_usage = {
            let mut pos = String::new();
            let mut noptional = 0;
            for pi in &self.positional {
                pos.push(' ');
                if !pi.is_required() {
                    pos.push('[');
                    noptional += 1;
                }
                pos.push_str(&pi.help_name);
                if pi.is_multi() {
                    pos.push_str("...");
                }
            }
            pos.push_str(&(0..noptional).map(|_| ']').collect::<String>());
            pos
        };
        let mut help = String::new();
        for pi_usage in self.positional.iter().map(PositionalItem::help) {
            help.push_str(&pi_usage)
        }
        help.push_str(&format!(
            "{0}-h  --help\\n\"\n\
             {0}      print this usage and exit\\n\"\n",
            HELP_PREFIX
        ));
        for npi_usage in self.non_positional.iter().map(NonPositionalItem::help) {
            help.push_str(&npi_usage)
        }
        format!(
            "static void usage(const char *progname) {{\n\
             \tprintf(\"usage: %s [options]{}\\n%s\", progname,\n\
             {}\t       );\n\
             }}\n",
            positional_usage, help
        )
    }
    /// Creates the parse_args function in C.
    fn cgen_decl(&self) -> String {
        let mut body = String::new();
        body.push_str("void parse_args(int argc, char **argv");
        for npi in &self.non_positional {
            body.push_str(&npi.cgen_decl_arg())
        }
        for pi in &self.positional {
            body.push_str(&pi.cgen_decl_arg())
        }
        body.push_str(") {\n");

        // decls for __isset
        for npi in &self.non_positional {
            body.push_str(&npi.cgen_isset_decl());
        }
        for pi in &self.positional {
            body.push_str(&pi.cgen_isset_decl());
        }
        // defs for __default
        for npi in &self.non_positional {
            body.push_str(&npi.cgen_default_decl());
        }
        for pi in &self.positional {
            body.push_str(&pi.cgen_default_decl());
        }

        // longopts
        // unique chars for each longopt
        let mut all_bytes: HashSet<u8> = (2..255).collect();
        // remove chars that are used for small opts
        for npi in &self.non_positional {
            if let Some(s) = &npi.short {
                all_bytes.remove(&s.as_bytes()[0]);
            }
        }
        let mut unused_bytes = all_bytes.drain().collect::<Vec<_>>();
        unused_bytes.sort();
        unused_bytes.reverse();
        let mut next_free_shortname = unused_bytes.into_iter();
        let uniqs: Vec<u8> = self
            .non_positional
            .iter()
            .map(|npi| {
                if let Some(s) = &npi.short {
                    s.as_bytes()[0]
                } else {
                    next_free_shortname
                        .next()
                        .expect("too many non-positional arguments")
                }
            })
            .collect();
        body.push_str("\tstatic struct option longopts[] = {\n");
        for (i, npi) in self.non_positional.iter().enumerate() {
            body.push_str(&npi.cgen_getopt(uniqs[i]));
        }
        body.push_str(
            "\t\t{\"help\", 0, 0, 'h'},\n\
             \t\t{0, 0, 0, 0}\n\t};\n",
        );

        // shortopts
        let mut optstring = String::from_utf8(
            self.non_positional
                .iter()
                .filter(|npi| npi.short.is_some())
                .flat_map(|npi| {
                    let s = npi.short.clone();
                    let mut v = Vec::new();
                    v.push(s.unwrap().as_bytes()[0]);
                    if !npi.is_flag() {
                        v.push(b':');
                    }
                    v.into_iter().collect::<Vec<u8>>()
                })
                .collect(),
        )
        .unwrap();
        optstring.push('h');

        // parse loop, optional
        body.push_str(&format!(
            "\tint ch;\n\
             \twhile ((ch = getopt_long(argc, argv, \"{}\", longopts, NULL)) != -1) {{\n\
             \t\tswitch (ch) {{\n",
            optstring
        ));
        for (i, uniq) in uniqs.iter().enumerate() {
            body.push_str(&format!(
                "\t\tcase {}:\n{}\t\t\tbreak;\n",
                uniq,
                self.non_positional[i].cgen_assign_optarg()
            ));
        }
        body.push_str(
            "\t\tcase 0:\n\t\t\tbreak;\n\
             \t\tcase 'h':\n\
             \t\tdefault:\n\t\t\tusage(argv[0]);\n\t\t\texit(1);\n\
             \t\t}\n\t}\n",
        );

        // post loop, optional
        for npi in &self.non_positional {
            body.push_str(&npi.cgen_post_loop());
        }

        // parse+post loop, positional
        let required: Vec<&PositionalItem> = self
            .positional
            .iter()
            .filter(|p| p.is_required() && !p.is_multi())
            .collect();
        let nrequired = required.len()
            + if self
                .positional
                .iter()
                .any(|p| p.is_required() && p.is_multi())
            {
                1
            } else {
                0
            };
        if nrequired > 0 {
            body.push_str(&format!(
                "\n\tif (argc-optind < {}) {{\n\
                   \t\tusage(argv[0]);\n\
                   \t\texit(1);\n\
                   \t}}\n\
                   \targv += optind;\n\targc -= optind;\n\n",
                nrequired
            ));
            if !required.is_empty() {
                for pi in &required {
                    body.push_str(&format!("{}\targv++;\n", pi.cgen_assign_argv0()));
                }
                if required.len() == 1 {
                    body.push_str("\targc--;\n\n");
                } else {
                    body.push_str(&format!("\targc -= {};\n\n", required.len()));
                }
                for pi in &required {
                    body.push_str(&pi.cgen_post_loop());
                }
            }
        }

        // parse+post loop, positional optional
        let optional: Vec<&PositionalItem> = self
            .positional
            .iter()
            .filter(|p| !p.is_required() && !p.is_multi())
            .collect();
        for pi in &optional {
            body.push_str("\tif (argc > 0) {\n");
            body.push_str(&pi.cgen_assign_argv0());
            body.push_str("\t\targv++; argc--;\n\t}\n");
        }
        for pi in &optional {
            body.push_str(&pi.cgen_post_loop());
        }

        // multi item
        let multi: Option<&PositionalItem> = self.positional.iter().find(|p| p.is_multi());
        if let Some(pi) = multi {
            if pi.is_required() {
                body.push_str(&pi.cgen_assign_argv0());
            } else {
                body.push_str("\tif (argc > 0) {\n");
                body.push_str(&pi.cgen_assign_argv0());
                body.push_str("\t}\n");
            }
            body.push_str(&pi.cgen_post_loop());
        }

        body.push_str("}\n");
        body
    }
    /// Creates the main function in C.
    fn cgen_main(&self) -> String {
        let mut main = String::new();
        main.push_str("int main(int argc, char **argv) {\n");

        for npi in &self.non_positional {
            main.push_str(&npi.cgen_main_decl())
        }
        for pi in &self.positional {
            main.push_str(&pi.cgen_main_decls())
        }

        main.push_str("\n\tparse_args(argc, argv");
        for npi in &self.non_positional {
            main.push_str(&npi.cgen_call_arg())
        }
        for pi in &self.positional {
            main.push_str(&pi.cgen_call_arg())
        }
        main.push_str(
            ");\n\n\
                      \t/* call your code here */\n\
                      \treturn 0;\n}\n",
        );
        main
    }
    /// Generates everything
    pub fn gen(&self) -> String {
        let h = self.cgen_headers();
        let usage = self.cgen_usage();
        let body = self.cgen_decl();
        let main = self.cgen_main();
        format!("{}\n\n{}\n{}\n{}", h, usage, body, main)
    }
    /// Writes generate C code to a writer.
    pub fn writeout<W>(&self, wrt: &mut W)
    where
        W: Write,
    {
        wrt.write_all(self.gen().as_bytes())
            .expect("write generated code to file")
    }
}
