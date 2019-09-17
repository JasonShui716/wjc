use std::collections::HashMap;
use std::error::Error;
// use std::ops::Range;

use rand;
use reqwest;
use scraper;

use digest::Digest as _;
use lazy_static::lazy_static;
use md5::Md5;
use sha1::Sha1;

const WUT_JWC: &str = "http://sso.jwc.whut.edu.cn/Certification//toIndex.do";
const WUT_JWC_GETCODE: &str = "http://sso.jwc.whut.edu.cn/Certification//getCode.do";
const WUT_JWC_LOGIN: &str = "http://sso.jwc.whut.edu.cn/Certification//login.do";

lazy_static! {
    static ref HEADERS: reqwest::header::HeaderMap = {
        use reqwest::header::*;
        let mut res = HeaderMap::new();
        macro_rules! insert_header {
            ($name: expr, $value: expr) => {
                res.insert($name, ($value).parse().unwrap());
            };
        }
        insert_header!(
            USER_AGENT,
            r#"Mozilla/5.0 (X11; Linux x86_64; rv:69.0) Gecko/20100101 Firefox/69.0"#
        );
        insert_header!(
            ACCEPT,
            r#"text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"#
        );
        insert_header!(ACCEPT_LANGUAGE, r#"zh-CN,en-US;q=0.5"#);
        insert_header!(ACCEPT_ENCODING, r#"gzip, deflate"#);
        insert_header!(DNT, r#"1"#);
        insert_header!(CONNECTION, r#"keep-alive"#);
        insert_header!(UPGRADE_INSECURE_REQUESTS, r#"1"#);
        res
    };
}

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Client {
        Client {
            client: reqwest::ClientBuilder::new()
                .default_headers(HEADERS.clone())
                .cookie_store(true)
                .build()
                .expect("Failed to build jwc client"),
        }
    }

    /// *Should be called once and only once*
    pub fn login(&self, username: &str, password: &str) -> Result<(), Box<dyn Error>> {
        let mut login_form = HashMap::new();

        // Empty fields
        login_form.insert("MsgID", String::from(""));
        login_form.insert("KeyID", String::from(""));
        login_form.insert("UserName", String::from(""));
        login_form.insert("Password", String::from(""));
        login_form.insert("return_EncData", String::from(""));

        // Hard-coded field
        login_form.insert("type", String::from("xs"));

        // Username and password
        login_form.insert("userName", String::from(username));
        login_form.insert("password", String::from(password));

        // Random fields
        login_form.insert("webfinger", generate_random_web_finger());
        login_form.insert(
            "rnd",
            scraper::Html::parse_document(&self.client.get(WUT_JWC).send()?.text()?)
                .select(&scraper::Selector::parse(r#"input[name="rnd"]"#).unwrap())
                .next()
                .expect("Failed to get rnd from html")
                .value()
                .attr("value")
                .expect("Failed to get value of rnd")
                .to_owned(),
        );

        // Get code field from another url
        login_form.insert(
            "code",
            self.client
                .post(WUT_JWC_GETCODE)
                .form(&[("webfinger", login_form.get("webfinger").unwrap())])
                .send()?
                .text()?,
        );

        // Calculate hashes
        login_form.insert("userName1", {
            let mut hasher = Md5::default();
            hasher.input(username);
            format!("{:x}", hasher.result())
        });
        login_form.insert("password1", {
            let mut hasher = Sha1::default();
            hasher.input(username);
            hasher.input(password);
            format!("{:x}", hasher.result())
        });

        self.client.post(WUT_JWC_LOGIN).form(&login_form).send()?;

        Ok(())
    }

    pub fn get_courses(&self) -> Result<String, Box<dyn Error>> {
        // Get data from grkbList.do
        self.client.get("http://218.197.102.183/Course").send()?;
        let grkb_list_html = self
            .client
            .get("http://218.197.102.183/Course/grkbList.do")
            .send()?
            .text()?;
        let grkb_list = scraper::Html::parse_document(&grkb_list_html);
        Ok(grkb_list
            .select(&scraper::Selector::parse(r#"[id="weekTable"]"#).unwrap())
            .next()
            .expect("No week course table")
            .html())
    }
}

fn generate_random_web_finger() -> String {
    let random_finger: u128 = rand::random();
    format!("{:x}", random_finger)
}

// pub struct Course {
//     name: String,
//     lessons: Vec<Lesson>,
// }

// pub struct Lesson {
//     teacher: String,
//     classroom: String,
//     week_number: usize,
//     stage: Range<usize>,
// }
