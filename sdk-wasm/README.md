## Known issues

### std library

Extending the wasm with other libraries can be strenuous. Since the wasm can not access to any OS resources, the rust std library has to be disabled. All dependencies must also not build on std.
If you face an error such as:

``
error: duplicate lang item in crate `[crate_name1]` (which `[crate_name2]` depends on): `[lang_item_name]`.
``

The crate_name1 is based on the std. A lot of time it is not obvious where the crate is used. With the command:


``
cargo tree -p [crate_name] -i --no-dedupe
``

You will see, where the crate is used.
