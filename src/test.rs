use super::*;

fn get_testdata_file_path(name: String) -> String {
    match std::env::current_dir() {
        Ok(mut exe_path) => {
            exe_path.push("testdata");
            exe_path.push(name);
            return exe_path.into_os_string().into_string().unwrap();
        }
        Err(e) => panic!("failed to get current exe path: {}", e),
    };
}

#[test]
fn read_file_ok() {
    let page_example = get_testdata_file_path("html_example".to_string());
    let content_file = read_file(&page_example);
    assert_eq!(content_file.len(), 449592);
}
//
//#[test]
//fn get_tag_links_ok() {
//    let page_example = get_testdata_file_path("html_example".to_string());
//    let content_file = read_file(&page_example);
//    let get = get_tag_links(&content_file);
//    let want = String::new();
//    println!("{}", get);
//    //    assert_eq!(get, want);
//}

#[test]
fn get_css_links_ok() {
    let (tx_stdout, rx_stdout) = mpsc::sync_channel(10000);

    let arg = String::from("background: url('https://mdn.mozillademos.org/files/16761/star.gif')");
    let want = String::from("https://mdn.mozillademos.org/files/16761/star.gif");
    get_css_links(&arg, &tx_stdout);
    let get = rx_stdout.recv().unwrap();
    assert_eq!(get, want);

    let arg = String::from("background: url(https://mdn.mozillademos.org/files/16761/star.gif)");
    let want = String::from("https://mdn.mozillademos.org/files/16761/star.gif");
    get_css_links(&arg, &tx_stdout);
    let get = rx_stdout.recv().unwrap();
    assert_eq!(get, want);

    let arg =
        String::from("background: url(\"https://mdn.mozillademos.org/files/16761/star.gif\")");
    let want = String::from("https://mdn.mozillademos.org/files/16761/star.gif");
    get_css_links(&arg, &tx_stdout);
    let get = rx_stdout.recv().unwrap();
    assert_eq!(get, want);
}

#[test]
fn get_src_links_ok() {
    let (tx_stdout, rx_stdout) = mpsc::sync_channel(10000);

    let arg = String::from("<img lowsrc=\"http://bl.com\">");
    let want = String::from("http://bl.com");
    get_src_links(&arg, &tx_stdout);
    let get = rx_stdout.recv().unwrap();
    assert_eq!(get, want);

    let arg = String::from("<img src=\"http://bl.com\">");
    let want = String::from("http://bl.com");
    get_src_links(&arg, &tx_stdout);
    let get = rx_stdout.recv().unwrap();
    assert_eq!(get, want);

    let arg = String::from("<img dynsrc=\"http://bl.com\">");
    let want = String::from("http://bl.com");
    get_src_links(&arg, &tx_stdout);
    let get = rx_stdout.recv().unwrap();
    assert_eq!(get, want);
}
#[test]
fn get_form_links_ok() {
    let (tx_stdout, rx_stdout) = mpsc::sync_channel(10000);

    let arg = String::from("<form action=\"/action_page.php\" method=\"get\">");

    let want = String::from("/action_page.php");
    get_form_links(&arg, &tx_stdout);
    let get = rx_stdout.recv().unwrap();
    assert_eq!(get, want);
}

#[test]
fn exist_url_ok() {
    let arg = String::from(
        "/da-dk/wp-json/oembed/1.0/embed?url=podd.time.com&par1=val1&par2=val2&par3=val3",
    );

    let want = true;
    let get = exist_url(&arg);
    assert_eq!(get, want);

    let arg = String::from(
        "/da-dk/wp-json/oembed/1.0/embed?rrr=vallll&url=podd.time.com&par1=val1&par2=val2&par3=val3",
    );

    let want = true;
    let get = exist_url(&arg);
    assert_eq!(get, want);

    let arg = String::from(
        "/da-dk/wp-json/oembed/1.0/embed?rrr=vallll&url=http://127.0.0.1&par1=val1&par2=val2&par3=val3",
    );

    let want = true;
    let get = exist_url(&arg);
    assert_eq!(get, want);

    let arg = String::from(
        "/da-dk/wp-json/oembed/1.0/embed?rrr=vallll&url=/path1/path2&par1=val1&par2=val2&par3=val3",
    );

    let want = true;
    let get = exist_url(&arg);
    assert_eq!(get, want);

    let arg = String::from(
        "/da-dk/wp-json/oembed/1.0/embed?rrr=vallll&url=/path1/path2&par1=val1&par2=val2&par3=val3",
    );

    let want = true;
    let get = exist_url(&arg);
    assert_eq!(get, want);

    let arg = String::from("/da-dk/wp-json/mail.com&2&par1=val1&par2=val2&par3=val3");

    let want = true;
    let get = exist_url(&arg);
    assert_eq!(get, want);
}

#[test]
fn clean_values_ok() {
    let arg=String::from("https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=https%3A%2F%2Fwww.etoro.com%2Fda-dk%2Ftrading%2Ffees%2F&par1=val1&par2=val2&par3=val3");

    let want =
        String::from("https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=&par1=&par2=&par3=");
    let get = clean_values(&arg);
    assert_eq!(get, want);

    let arg = String::from(
        "https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=&par1=val1&par2=val2&par3=val3",
    );
    let want =
        String::from("https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=&par1=&par2=&par3=");
    let get = clean_values(&arg);
    assert_eq!(get, want);

    let arg = String::from(
        "https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=&par1=ssssss&par2=&par3=oooo",
    );
    let want =
        String::from("https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=&par1=&par2=&par3=");
    let get = clean_values(&arg);

    let arg = String::from(
        "https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=&par1=s#%1sssss&par2=sss%1sssss&par3=ooo!@#$%^*()o",
    );
    let want =
        String::from("https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=&par1=&par2=&par3=");
    let get = clean_values(&arg);
    assert_eq!(get, want);
}

#[test]
fn decode_url_ok() {
    let arg=String::from("https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=https%3A%2F%2Fwww.etoro.com%2Fda-dk%2Ftrading%2Ffees%2F");

    let want =
        String::from("https://www.etoro.com/da-dk/wp-json/oembed/1.0/embed?url=https://www.etoro.com/da-dk/trading/fees/");
    let get = decode_url(arg);
    assert_eq!(get, want);
}

#[test]
fn remove_domain_1() {
    let arg = String::from("https://www.etoro.com/en/markets/MIOTA?funnelfromid=57");

    let want_path = String::from("/en/markets/MIOTA?funnelfromid=57");
    let want_domain = String::from("https://www.etoro.com");
    let (get_url, get_domain) = remove_domain(&arg);

    assert_eq!(get_url, want_path);
    assert_eq!(get_domain, want_domain);
}

#[test]
fn remove_domain_without_params() {
    let arg = String::from("https://www.etoro.com/nl/people/YveLemkens");

    let want_path = String::from("/nl/people/YveLemkens");
    let want_domain = String::from("https://www.etoro.com");
    let (get_url, get_domain) = remove_domain(&arg);

    assert_eq!(get_url, want_path);
    assert_eq!(get_domain, want_domain);
}

#[test]
fn remove_domain_without_www() {
    let arg = String::from("https://accessibe.com?utm_medium=link&utm_source=widget");

    let want_url = String::from("/?utm_medium=link&utm_source=widget");
    let want_domain = String::from("https://accessibe.com");
    let (get_url, get_domain) = remove_domain(&arg);

    assert_eq!(get_url, want_url);
    assert_eq!(get_domain, want_domain);
}

#[test]
fn remove_domain_unknown() {
    let arg = String::from("data:https://www.youtube.com/embed/7O4rP3qwlfo");

    let want_url = String::from("data:https://www.youtube.com/embed/7O4rP3qwlfo");
    let want_domain = String::from("");
    let (get_url, get_domain) = remove_domain(&arg);

    assert_eq!(get_url, want_url);
    assert_eq!(get_domain, want_domain);
}

#[test]
fn remove_domain_without_base() {
    let arg = String::from("//www.etoro.com/accounts/sign-up");

    let want_path = String::from("//www.etoro.com/accounts/sign-up");
    let want_domain = String::new();
    let (get_path, get_domain) = remove_domain(&arg);

    assert_eq!(get_path, want_path);
    assert_eq!(get_domain, want_domain);
}
#[test]
fn remove_domain_without_domain_ok() {
    let arg=String::from("/ext/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c");

    let want_url = String::from("/ext/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c");
    let want_domain = String::new();
    let (get_url, get_domain) = remove_domain(&arg);

    assert_eq!(get_url, want_url);
    assert_eq!(get_domain, want_domain);
}

#[test]
fn remove_domain_ok() {
    let arg = String::from("http://somedomain.com/ext/eyJhbG?par1=val1&par2=val2");

    let want_url = String::from("/ext/eyJhbG?par1=val1&par2=val2");
    let want_domain = String::from("http://somedomain.com");

    let (get_url, get_domain) = remove_domain(&arg);

    assert_eq!(get_url, want_url);
    assert_eq!(get_domain, want_domain);
}

#[test]
fn decode_jwt_ok() {
    let arg=String::from("http://somedomain.com/ext/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c");

    let want =
        String::from("http://somedomain.com/ext/{\"alg\":\"HS256\",\"typ\":\"JWT\"}.{\"sub\":\"1234567890\",\"name\":\"John Doe\",\"iat\":1516239022}.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c");
    let get = decode_base64(arg);
    assert_eq!(get, want);
}

#[test]
fn decode_base64_ok() {
    let arg=String::from("http://slavyoga.ru/ext/aHR0cHM6Ly9tdXNpYy55YW5kZXgucnUvYWxidW0vMTgxMDkzL3RyYWNrLzU4MDczMw==");

    let want =
        String::from("http://slavyoga.ru/ext/https://music.yandex.ru/album/181093/track/580733");
    let get = decode_base64(arg);
    assert_eq!(get, want);
}

#[test]
fn decode_base64_as_param() {
    let arg = String::from(
        "https://www.base64encode.org?somepar=aHR0cHM6Ly93d3cuYmFzZTY0ZW5jb2RlLm9yZw==",
    );

    let want = String::from("https://www.base64encode.org?somepar=https://www.base64encode.org");
    let get = decode_base64(arg);
    assert_eq!(get, want);
}

#[test]
fn decode_base64_not_found() {
    let arg = String::from(
        "http://slavyoga.ru/ext/aH6Ly9tdXNpYy55YW5kZXgucnUvYWxidW0vMTgxMDkzL3RyYWNrLzU4MDczMw==",
    );

    let want =
        String::from("http://slavyoga.ru/ext/https://music.yandex.ru/album/181093/track/580733");
    let get = decode_base64(arg);
    assert_ne!(get, want);
}

#[test]
fn get_all_links_ok() {
    let (tx_stdout, rx_stdout) = mpsc::sync_channel(10000);
    let arg = String::from(
        "background-image:url(https://etoro-cdn.etorostatic.com/studio/content/lp/cache_1/etoro-lps/general_images/cryptocurrencies-circles/\"ethereum-classic.svg\");-webkit-animation-delay:-8s;animation-delay:-8s;background-size:60%}.circles-group[_ngcontent-eToro-LPs-c5",
    );

    let want =
        String::from("https://etoro-cdn.etorostatic.com/studio/content/lp/cache_1/etoro-lps/general_images/cryptocurrencies-circles/ethereum-classic.svg");
    let get = get_all_links(&arg, tx_stdout);
    let get = rx_stdout.recv().unwrap();
    assert_eq!(get, want);
}
