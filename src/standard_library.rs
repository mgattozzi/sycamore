use wasmtime::*;

pub fn println(store: &mut Store<()>) -> Func {
  Func::wrap(
    store,
    |mut caller: Caller<'_, ()>, offset: i32, len: i32| {
      let data = &caller
        .get_export("main_memory")
        .unwrap()
        .into_memory()
        .unwrap()
        .data(caller.as_context());
      println!(
        "{}",
        std::str::from_utf8(&data[offset as usize..len as usize]).unwrap()
      );
    },
  )
}

pub fn print(store: &mut Store<()>) -> Func {
  Func::wrap(
    store,
    |mut caller: Caller<'_, ()>, offset: i32, len: i32| {
      let data = &caller
        .get_export("main_memory")
        .unwrap()
        .into_memory()
        .unwrap()
        .data(caller.as_context());
      print!(
        "{}",
        std::str::from_utf8(&data[offset as usize..len as usize]).unwrap()
      );
    },
  )
}
