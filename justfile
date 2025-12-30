# Run an example with hot reload
watch example:
    cargo watch -x 'run --example {{ example }}'

# Run an example with hot reload in release mode
watch-release example:
    cargo watch -x 'run --example {{ example }} --release'

# Run an example normally
run example:
    cargo run --example {{ example }}

# List all available examples
list-examples:
    @ls -1 crates/astra-gui-wgpu/examples/*.rs | xargs -n1 basename | sed 's/\.rs$//'

# Short aliases

alias w := watch
alias wr := watch-release
alias r := run
alias ls := list-examples
