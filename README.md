# argen

**`argen`** lets you generate argument parsing logic in C from a simple
config. No more having to write that tedious arg-parsing C code!

![argen](examples/example.png)

```c
// this is what the entry point looks like from the above example

int main(int argc, char **argv) {
	char* out_file;
	char* in_file;
	char* *words;
	size_t words__size;
	int block_size;
	int fave_number;
	int quiet;
	char* username;

	parse_args(argc, argv, &out_file, &in_file, &words, &words__size, &block_size, &fave_number, &quiet, &username);

	/* call your code here */
}
```

## Installation 

#### Standalone

We have not yet released precompiled binaries for **`argen`**.

#### Source

```sh
# install rustup.rs
$ curl https://sh.rustup.rs -sSf | sh
# clone the source code
$ git clone https://github.com/kynelee/argen.git
$ cd argen
# build
$ cargo build --release
# copy binary
$ cp target/release/argen /usr/local/bin/argen
```

## Usage 

To generate a C entry point, run 
`some_command specs.toml`

The spec.toml file specifies how you want your C code to parse arguments. 

```toml 
# this is an example spec.toml which argen uses 
# to generate C argument parsing code.
# example usage: ./generated_program --set-flag --block-size 10 file1.txt file2.txt 

# Corresponds to --set-flag argument
[[non_positional]]
c_var = "flag_set"                  # required, variable name in C codeb
c_type = "int"                      # required, variable type in C 
flag = true                         # optional, no values are passed into this argument
help_name = "flag option"           # required, display name on the -help output
help_descr = "Set true or false"    # required, description on the -help output


# Corresponds to --block-size argument and its value, 10.
[[non_positional]] 
c_var = "block_size" 
c_type = "int"
long = "block-size"                 # required, specifies argument name
short = "b"                         # optional, shortcut for argument name, 1 ASCII character only
aliases = ["bs"]                    # optional, more aliases for argument name
default = "12"                      # optional, default value for variable 
help_name = "block size"
help_descr = "Set the block size"


# Corresponds to first positional argument, input_file
[[positional]]
c_var  = "input_file"
c_type = "char*"
required = "true"                   # optional, defaults to false. Produces error if input_file is not passed
help_name = "input file"
help_descr = "input file, required"


# Corresponds to the second positional argument, output_file 
[[positional]]
c_var = "output_file"
c_type = "char*"
help_name = "output file"
help_descr = "file for output"
default = "output.txt"
```

After generating and compiling the C code, you should have a fully
functioning C argument parsing code.

Feel free to check out out this [example TOML file](examples/api.toml) which details all the configuration
options available, or some other [TOML configuration examples](/examples/).
