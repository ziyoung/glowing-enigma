use std::{io::Read, time::Duration};

use once_cell::sync::Lazy;
use regex::Regex;
use tikv_jemallocator::Jemalloc;
use tokio::fs;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

static REG: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(Object\.\w+)"#).unwrap());

async fn dump_prof(id: usize) -> anyhow::Result<()> {
    let mut prof_ctl = jemalloc_pprof::PROF_CTL.as_ref().unwrap().lock().await;
    let mut file = prof_ctl
        .dump()
        .map_err(|e| anyhow::anyhow!("load .prof file failed: {}", e))?;
    let mut buf = Vec::with_capacity(1024 * 16);
    file.read_to_end(&mut buf)
        .map_err(|e| anyhow::anyhow!("write prof file to buffer failed: {}", e))?;
    let name = format!("./profile/regex-captures-iter-{}.prof", id);
    fs::write(name, buf)
        .await
        .map_err(|e| anyhow::anyhow!("write .prof file failed: {}", e))
}

fn get_tag_positions(src: &str) -> Vec<(usize, usize)> {
    REG
        .clone()
        .captures_iter(src)
        .map(|caps| {
            let m = caps.get(0).unwrap();
            (m.start(), m.end())
        })
        .collect::<Vec<_>>()
}

async fn memory_leak_demo() -> Vec<u8> {
    let text = reqwest::get("https://cdnjs.cloudflare.com/ajax/libs/vue/3.4.34/vue.cjs.js")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let mut buf = Vec::from(text.as_bytes());
    for (i, (start, end)) in get_tag_positions(&text).into_iter().enumerate() {
        println!("process text: {}", i + 1);
        tokio::time::sleep(Duration::from_millis(50)).await;
        // let m = caps.get(0).unwrap();
        // let start = m.start();
        // let end = m.end();
        let text = &text[start..end];
        buf.extend(text.as_bytes());
    }
    buf
}

async fn task() {
    for _ in 0..50 {
        let buf = memory_leak_demo().await;
        let s = String::from_utf8(buf).unwrap_or_default();
        println!("string length: {}", s.len());
    }
}

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        dump_prof(0).await.unwrap();
        for i in 0..5 {
            task().await;
            tokio::time::sleep(Duration::from_secs(10)).await;
            println!("write prof");
            dump_prof(i + 1).await.unwrap();
        }
    })
    .await
    .unwrap();
    println!("done");
}
