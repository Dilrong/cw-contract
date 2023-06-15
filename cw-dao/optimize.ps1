docker run --rm -v ${PWD}:/code `
 -v "$(Split-Path -Path ${PWD} -Leaf)_cache:/code/target" `
 -v registry_cache:/usr/local/cargo/registry `
 cosmwasm/rust-optimizer:0.12.13