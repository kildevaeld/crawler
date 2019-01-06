use duktape::error as dukerr;
use duktape_modules::error as dukcjserr;
use reqwest::Error as ReqError;
use std::io::Error as IOError;
use url;

error_chain! {
    foreign_links {
        Url(url::ParseError);
        Reqwest(ReqError);
        IO(IOError);
    }

    links {
        DukError(dukerr::Error, dukerr::ErrorKind);
        DukCJSError(dukcjserr::Error, dukcjserr::ErrorKind);
    }

    errors {
        Http(code:u16) {
            description("http error")
            display("http error: {}", code)
        }
    }
}
