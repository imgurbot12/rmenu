mod bang;

use std::io::{BufRead, BufReader, Read};

use rmenu_plugin::{Entry, Message, Search};

use crate::bang::Bang;

static DEFAULT_BANG: &'static str = "!brave";

fn send_entry(entry: &Entry) {
    let message = serde_json::to_string(&entry).expect("invalid entry");
    println!("{message}");
}

fn read_search<T: Read>(reader: &mut BufReader<T>) -> Option<Search> {
    let mut buf = String::new();
    match reader.read_line(&mut buf) {
        Ok(_) => {}
        Err(err) => {
            log::error!("failed to read stdin: {err:?}");
            return None;
        }
    }
    buf = buf.trim().to_owned();
    if buf.is_empty() {
        log::debug!("message empty. breaking loop.");
        return None;
    }
    serde_json::from_str(&buf).ok()
}

fn main() {
    env_logger::init();

    let bangs = Bang::bangs();
    let rgx = regex::RegexBuilder::new(r"!\w+")
        .build()
        .expect("bang regex failed");

    let mut fend = fend_core::Context::new();

    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        // read search message from stdin
        let Some(search) = read_search(&mut reader) else {
            continue;
        };

        // retrieve bang based on search
        let search_bang = rgx.find(&search.search).map(|m| m.as_str());

        // check if search can be processed by fend
        if let Ok(result) = fend_core::evaluate(&search.search, &mut fend) {
            let calc = result.get_main_result();
            if !calc.is_empty() {
                let action = format!("wl-copy {calc:?}");
                let entry = Entry::new(&calc, &action, None);
                send_entry(&entry);
            }
        }

        // retrieve bang based on search
        let search_bang = search_bang.unwrap_or(DEFAULT_BANG);
        let bang = match bangs.get(&search_bang[1..]) {
            Some(bang) => bang,
            None => {
                log::warn!("invalid bang: {search_bang:?}");
                bangs.get(DEFAULT_BANG).expect("default bang missing")
            }
        };

        // remove bang from search string
        let query = search.search.replace(search_bang, "");
        let query = query.trim();

        let name = format!("{} - {query}", bang.name);

        let action = format!("xdg-open {:?}", bang.url.replace(r"{{{s}}}", query));
        let entry = Entry::new(&name, &action, None);
        send_entry(&entry);

        let stop = Message::Stop;
        let message = serde_json::to_string(&stop).expect("invalid stop");
        println!("{message}");
    }
}
