use abi_stable::std_types::{RHashMap, RString};
use rmenu_plugin::internal::load_plugin;

static PLUGIN: &str = "../../plugins/run/target/release/librun.so";

fn test() {
    let mut cfg = RHashMap::new();
    // cfg.insert(RString::from("ignore_case"), RString::from("true"));

    let mut plugin = unsafe { load_plugin(PLUGIN, &cfg).unwrap() };
    let results = plugin.module.search(RString::from("br"));
    for result in results.into_iter() {
        println!("{} - {:?}", result.name, result.comment);
    }
    println!("ayy lmao done!");
}

fn main() {
    test();
}
