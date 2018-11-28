use duktape::error as dukerr;
use duktape_cjs::error as dukcjserr;
use url;

error_chain!{
    foreign_links {
        Url(url::ParseError);
    }

    links {
        DukError(dukerr::Error, dukerr::ErrorKind);
        DukCJSError(dukcjserr::Error, dukcjserr::ErrorKind);
    }
}
