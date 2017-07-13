use std;
use std::io::Read;
use std::fmt;

use futures::BoxFuture;

extern crate futures_cpupool;
use self::futures_cpupool::CpuPool;

use gltf::import::{Source};

extern crate reqwest;

pub struct HttpSource {
    url: reqwest::Url,
    cpu_pool: CpuPool,
}

impl fmt::Debug for HttpSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HttpSource {{ url: {} }}", self.url)
    }
}

impl HttpSource {
    pub fn new(url: &str) -> HttpSource {
        // 6 threads - like max parallel requests per domain in browsers
        let pool = CpuPool::new(6);

        HttpSource {
            url: reqwest::Url::parse(url)
                .expect("Failed to parse URL"),
            cpu_pool: pool
        }
    }
}

#[derive(Debug)]
pub enum Error {
    HttpError(String),
}

impl HttpSource {
    fn fetch_data(&self, url: String) -> BoxFuture<Box<[u8]>, Error> {
        let future = self.cpu_pool.spawn_fn(move || {
            let mut resp = reqwest::get(&url).unwrap();
            // TODO: return error instead
            assert!(resp.status().is_success(), "request failed: {}", resp.status());
            // TODO: status not showing on console...
            // if !resp.status().is_success() {
            //     return Err(Error::HttpError(format!("{}", resp.status())));
            // }
            let mut data = vec![];
            let _ = resp.read_to_end(&mut data);
            Ok(data.into_boxed_slice())
        });
        Box::new(future)
    }
}

impl Source for HttpSource {
    type Error = Error;
    fn source_gltf(&self) -> BoxFuture<Box<[u8]>, Self::Error> {
        self.fetch_data(self.url.to_string())
    }

    fn source_external_data(&self, uri: &str) -> BoxFuture<Box<[u8]>, Self::Error> {
        let mut new_url = self.url.clone();
        new_url.path_segments_mut()
            .expect("URL cannot be base")
            .pop().push(uri);
        self.fetch_data(new_url.to_string())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "HttpSource Error"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        unimplemented!() // TODO
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        write!(f, "{}", self.description())
    }
}
