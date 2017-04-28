# argen

**`argen`** lets you generate argument parsing logic in C from a simple
config. No more having to write that tedious arg-parsing C code!

![argen](examples/example.png)

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

A simple command line interface you want to build might resemble this: 

`./a.out --arg1 1 --arg2 c --quiet "positional_arg" "another_positional_arg`

To generate the corresponding C code to parse this interface you must create a TOML file which
describes how your CLI works.

The corresponding TOML file to generate the C code to implement the above CLI is this 

```
Inline spec
Include Comment Descriptions 
```

Now, we can generate the C code using this TOML file and argen.

```sh
argen -o main.c spec.toml
```

Check out the generated code. You'll notice that all the C variables
corresponding to the value of our command line arguments 
are declared according to the name and type in the TOML file. After the call to `method_name`,
all your variables are properly initialized and ready to be used however
you want. 

When you compile the C code and run the executable, you'll notice the help and usage dialogues 
displayed in the command line.

In other words, you've built a fully functioning CLI with 0 programming logic!

Argen also supports features which allow you to create much more complex  
CLIs. 

Imagine if you wanted to create a CLI like this: 
`/executable --set-flag --def-arg "input_file" --required-arg 2 --optional-arg "maybe" opt_p_arg req_p_arg` 

Here, we have multiple different argument types which we want to pass into the program. 
An arg type like `--set-flag` requires no value and serves as a flag. Arguments  
like `--def-arg` should default to some value if not specified. 
Other arguments like `--required-arg` are required and will error if no values are passed in. 

Writing the code for this is boring and tedious. Using argen, you
can create this by simply specifying more options in the TOML file. 

Feel free to check out out this [example TOML file](examples/example_spec.toml) which details all the configuration
options available, or some [fully functioning examples](/examples/).
