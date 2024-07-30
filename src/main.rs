use std::{env, time::Duration};

use once_cell::sync::Lazy;
use regex::Regex;
use tokio::{fs, task::JoinSet};

static REG: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(Object\.\w+)"#).unwrap());

async fn memory_leak_demo() -> Vec<u8> {
    let file = fs::read("./vue.js").await.unwrap();
    let mut buf = file.clone();
    let text = String::from_utf8(file).unwrap();

    if env::var("SYNC_REG_MATCH").is_ok() {
        let positions = REG
            .captures_iter(&text)
            .map(|caps| {
                let m = caps.get(0).unwrap();
                (m.start(), m.end())
            })
            .collect::<Vec<_>>();
        for (start, end) in positions {
            buf.extend(text[start..end].as_bytes());
        }
    } else {
        for caps in REG.captures_iter(&text) {
            tokio::time::sleep(Duration::from_millis(5)).await;
            let m = caps.get(0).unwrap();
            let start = m.start();
            let end = m.end();
            let text = &text[start..end];
            buf.extend(text.as_bytes());
        }
    }
    buf
}

#[tokio::main]
async fn main() {
    {
        tokio::spawn(async {
            for i in 0..20 {
                let mut set = JoinSet::new();
                for _ in 0..10_000 {
                    set.spawn(memory_leak_demo());
                }
                while let Some(_res) = set.join_next().await {}
                println!("task {} done", i);
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
    }
    println!("use CTRL + C to exit");
    tokio::signal::ctrl_c().await.unwrap();
}
