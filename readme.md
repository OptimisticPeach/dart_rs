# `dart`
**Idiomatic Bindings to the Dart Native Extensions API**

This crate serves three purposes: 
- To provide a kind of "filter" between what's unsafe to do and what is safe to do. 
  These items are found directly within `/src`, and are essentially light wrappers 
  around the raw api.
- To provide idiomatic bindings to some core Dart types. These are found under 
  `/src/dart_types`, and provide rust-like semantics for dart operations, operators,
   functions, etc.
- To make integration with the Dart VM easier, by providing macros which generate 
  functions with the correct names, attributes and boilerplate to create a native
  extension function. These macros are the `export_dart_functions` and
  `create_init_function`.
  
Usage of the macros to expose correct functionality can be seen in the examples, with
both the async and sync functions being exposed. 

Note, that the creation of libraries using this crate must undergo the same process
as described in [`dart_sys`'s readme](https://github.com/optimisticpeach/dart-sys).
