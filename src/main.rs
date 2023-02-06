#![allow(unused)]
extern crate base64;
extern crate crossbeam_utils;
extern crate html_escape;
extern crate num_cpus;
extern crate regex;

use crossbeam_utils::sync::WaitGroup;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead, BufReader};
use std::slice::SliceIndex;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, sleep};
use std::time::Duration;
use std::vec::Vec;
use url::{ParseError, Url};
use urlencoding::decode;

fn main() {
    let (tx, rx) = mpsc::channel();
    let (tx_stdout, rx_stdout) = mpsc::sync_channel(10000);
    let rx = Arc::new(Mutex::new(rx));
    let cpus = num_cpus::get();

    let wg_stdout = WaitGroup::new();
    let wg_stdout_child = wg_stdout.clone();
    rayon::spawn(move || stdout_worker(wg_stdout_child, rx_stdout));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cpus)
        .build()
        .unwrap();

    let wg = WaitGroup::new();
    for num_th in (0..cpus) {
        let receiver = Arc::clone(&rx);
        let tx_stdout = tx_stdout.clone();
        let wg = wg.clone();
        pool.spawn(move || worker(num_th, wg, receiver, tx_stdout))
    }

    for result in io::stdin().lock().lines() {
        match result {
            Ok(file_path) => {
                tx.send(file_path).unwrap();
            }
            Err(err) => continue,
        }
    }
    drop(tx);
    wg.wait();
    drop(tx_stdout);
    wg_stdout.wait();
}

fn decode_base64(s: String) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"((?:eyJ|YTo|Tzo|PD[89]|aHR0cHM6L|aHR0cDo|rO0)[%a-zA-Z0-9+/]+={0,2})"#)
            .unwrap()
    });

    let mut result = String::new();
    result = s.clone();
    for caps in RE.captures_iter(&s) {
        let encode_base64_str = caps.get(1).unwrap().as_str().to_string();
        if encode_base64_str.len() == 0 {
            continue;
        }

        match base64::decode(&encode_base64_str) {
            Ok(buf) => match String::from_utf8(buf) {
                Ok(decode_base64_str) => {
                    result = result.replace(encode_base64_str.as_str(), &decode_base64_str);
                }
                Err(e) => return s,
            },
            Err(e) => return s,
        }
    }
    return result;
}

fn decode_url(mut url: String) -> String {
    if url.contains("%") {
        match decode(&url) {
            Ok(url) => return url.into_owned(),
            Err(e) => return url,
        }
    }
    return url;
}

fn remove_domain(url: &String) -> (String, String) {
    let mut url_child_str = url.clone();
    let url_entity = Url::parse(url.as_str());
    let mut domain = String::new();
    let mut path = String::new();

    match url_entity {
        Ok(url) => {
            path = url.path().to_string();
            match url.domain() {
                Some(d) => {
                    domain = d.to_string();
                }
                None => {
                    return (url_child_str, domain);
                }
            }

            let mut base = url.scheme().to_string();
            base.push_str("://");
            base.push_str(&domain);
            match url.query().to_owned() {
                Some(q) => {
                    path.push_str("?");
                    path.push_str(q)
                }
                None => {}
            }
            (path, base)
        }
        Err(e) => (url_child_str, domain),
    }
}

fn map_as_string(m: &HashMap<String, String>) -> String {
    let s = m.values().map(|s| &**s).collect::<Vec<_>>().join(&"\n");
    return s;
}

fn clean_values(url: &String) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"=([^&]+)"#).unwrap());

    let mut result = url.clone();
    for caps in RE.captures_iter(&url) {
        let val = caps.get(1).unwrap().as_str().to_string();
        if val.len() == 0 {
            continue;
        }

        result = result.replace(val.as_str(), "");
    }

    return result;
}

fn is_social_network(url: &String) -> bool {
    static RE_SOCIAL_NET: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"https?://[-\d\w\.]*(?:googleads|adalyser\.com|rfihub\.com|:fls\.doubleclick\.net|t\.co|cloudfront\.net|cloudfront\.com|cloudinary\.com|bing\.com|addthis\.com|cheqzone\.com|nr-data.net|analytics\.yahoo\.com|taboola\.com|steelhousemedia\.com|linkedin\.com|youtube\.com|www\.facebook\.com|facebook\.com|twitter\.com|instagram\.com|mail.ru|vk.com|yandex\.ru|google\.\w+|googleapis\.com|ok\.ru|googletagmanager\.com|google-analytics\.com)"#).unwrap()
    });

    if RE_SOCIAL_NET.is_match(&url) {
        return true;
    }

    false
}
fn exist_url(path: &String) -> bool {
    if path.contains("http") {
        return true;
    }

    if path.contains("www.") {
        return true;
    }

    static RE_RELATIVE_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r#"=/"#).unwrap());
    if RE_RELATIVE_URL.is_match(&path) {
        return true;
    }
    // можно отдельно обработать параметры в ссылках
    static RE_DOMAIN: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
			r#"(?:[^\.]+)(?:\.com|\.net|\.org|\.info|\.coop|\.int|\.co\.uk|\.org\.uk|\.ac\.uk|\.au|\.ca|\.com\.au|\.net\.au|\.org\.au|\.org\.au|\.id\.au|\.biz)"#,
		)
			.unwrap()
    });
    if RE_DOMAIN.is_match(&path) {
        return true;
    }

    return false;
}

fn is_trash(url: &String) -> bool {
    static RE_RELATIVE_URL: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?:^//|blob:|mailto:|\.png$|\.svg$|\.mp4$|\.js|\.jpg$|\.aspx$|\.php$)"#)
            .unwrap()
    });
    if RE_RELATIVE_URL.is_match(&url) {
        return true;
    }
    false
}

fn stdout_worker(wg: WaitGroup, receiver: mpsc::Receiver<String>) {
    let mut uniq_buf: HashMap<String, String> = HashMap::new();
    for url in receiver.iter() {
        if is_trash(&url) || is_social_network(&url) {
            continue;
        }
        let (mut path, mut domain) = remove_domain(&url);
        path = decode_base64(path);
        path = decode_url(path);
        path = html_escape::decode_html_entities(&path).into_owned();
        if !exist_url(&path) {
            continue;
        }
        domain.push_str(path.as_str());

        let key = clean_values(&url);
        uniq_buf.insert(key, domain);
    }

    println!("{}", map_as_string(&uniq_buf));
}

fn worker(
    num_th: usize,
    wg: WaitGroup,
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
    tx_stdout: mpsc::SyncSender<String>,
) {
    for file_path in receiver.lock().unwrap().iter() {
        let tx_stdout_child = tx_stdout.clone();
        let file_content = read_file(&file_path);
        get_tag_links(&file_content, tx_stdout_child);
        //        let tx_stdout_child = tx_stdout.clone();
        //        get_all_links(&file_content, tx_stdout_child);
    }
}

fn get_all_links(file_content: &String, tx_stdout: mpsc::SyncSender<String>) {
    static RE_ALL_LINKS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(https?:\\/\\/.*?)"#).unwrap());

    for caps in RE_ALL_LINKS.captures_iter(&file_content) {
        let url = caps.get(1).unwrap().as_str().to_string();
        if url.len() == 0 {
            continue;
        }
        tx_stdout.send(url);
    }
}

fn read_file(path: &String) -> String {
    let result = File::open(path);
    match result {
        Ok(f) => {
            let mut buf_str = String::new();
            let mut buf_reader = BufReader::new(f);
            let result = buf_reader.read_to_string(&mut buf_str);
            match result {
                Ok(_) => buf_str,
                Err(err) => {
                    return err.to_string();
                }
            }
        }
        Err(err) => panic!("critical_err: can't open the file {}, {:?}", path, err),
    }
}

fn get_tag_links(file_content: &String, tx_stdout: mpsc::SyncSender<String>) {
    get_href_links(&file_content, &tx_stdout);
    get_src_links(&file_content, &tx_stdout);
    get_form_links(&file_content, &tx_stdout);
    get_css_links(&file_content, &tx_stdout);
    get_html4_links(&file_content, &tx_stdout);
    //    get_srcset_links(&file_content, &tx_stdout);
    get_html5_links(&file_content, &tx_stdout);
}

fn get_srcset_links(file_content: &String, tx_stdout: &mpsc::SyncSender<String>) {
    static RE_SRCSET_LINK: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"srcset=["'](https?://.+?) ["']"#).unwrap());

    for caps in RE_SRCSET_LINK.captures_iter(&file_content) {
        let url = caps.get(1).unwrap().as_str().to_string();
        if url.len() == 0 {
            continue;
        }
        tx_stdout.send(url);
    }
}

fn get_html5_links(file_content: &String, tx_stdout: &mpsc::SyncSender<String>) {
    static RE_HTML5_LINK: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?:profile|data|dsync|formaction|icon|manifest|poster)=["'](.+?)["']"#)
            .unwrap()
    });

    for caps in RE_HTML5_LINK.captures_iter(&file_content) {
        let url = caps.get(1).unwrap().as_str().to_string();
        if url.len() == 0 {
            continue;
        }
        tx_stdout.send(url);
    }
}

fn get_html4_links(file_content: &String, tx_stdout: &mpsc::SyncSender<String>) {
    static RE_HTML4_LINK: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?:codebase|cite|background|longdesc|usemap|archive|classid)=["'](.+?)["']"#)
            .unwrap()
    });

    for caps in RE_HTML4_LINK.captures_iter(&file_content) {
        let url = caps.get(1).unwrap().as_str().to_string();
        if url.len() == 0 {
            continue;
        }
        tx_stdout.send(url);
    }
}

fn get_css_links(file_content: &String, tx_stdout: &mpsc::SyncSender<String>) {
    static RE_CSS_LINK: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"url\(["']?(http?.+?)["']?\)"#).unwrap());

    for caps in RE_CSS_LINK.captures_iter(&file_content) {
        let url = caps.get(1).unwrap().as_str().to_string();
        if url.len() == 0 {
            continue;
        }
        tx_stdout.send(url);
    }
}

fn get_form_links(file_content: &String, tx_stdout: &mpsc::SyncSender<String>) {
    static RE_FORM_LINK: Lazy<Regex> = Lazy::new(|| Regex::new(r#"action=["'](.+?)["']"#).unwrap());

    for caps in RE_FORM_LINK.captures_iter(&file_content) {
        let url = caps.get(1).unwrap().as_str().to_string();
        if url.len() == 0 {
            continue;
        }
        tx_stdout.send(url);
    }
}

fn get_href_links(file_content: &String, tx_stdout: &mpsc::SyncSender<String>) {
    static RE_HREF_LINK: Lazy<Regex> = Lazy::new(|| Regex::new(r#"href=["'](.+?)["']"#).unwrap());

    for caps in RE_HREF_LINK.captures_iter(&file_content) {
        let url = caps.get(1).unwrap().as_str().to_string();
        if url.len() == 0 {
            continue;
        }
        tx_stdout.send(url);
    }
}

fn get_src_links(file_content: &String, tx_stdout: &mpsc::SyncSender<String>) {
    static RE_SRC_LINK: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"(?:src|lowsrc|dynsrc)=["'](.+?)["']"#).unwrap());

    for caps in RE_SRC_LINK.captures_iter(&file_content) {
        let url = caps.get(1).unwrap().as_str().to_string();
        if url.len() == 0 {
            continue;
        }
        tx_stdout.send(url);
    }
}

#[cfg(test)]
mod test;
