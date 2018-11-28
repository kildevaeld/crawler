use super::error::Result;
use duktape::prelude::*;
use duktape::types::{FromDuktape, ToDuktape};
use duktape::{
    error::Error as DukError, error::ErrorKind as DukErrorKind, error::Result as DukResult,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Work {
    Path(Option<PathBuf>, String),
}

impl Work {
    pub fn path<T: AsRef<Path>, S: AsRef<str>>(root: Option<T>, path: S) -> Work {
        let root = match root {
            Some(root) => Some(root.as_ref().to_path_buf()),
            None => None,
        };
        Work::Path(root, path.as_ref().to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Task {
    id: Uuid,
    #[serde(with = "url_serde")]
    url: Url,
    work: Work,
}

impl Task {
    pub fn new<T: AsRef<str>>(url: T, work: Work) -> Result<Task> {
        let url = Url::parse(url.as_ref())?;
        let id = Uuid::new_v4();
        Ok(Task { work, url, id })
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn set_url<T: AsRef<Url>>(&mut self, url: T) -> &mut Self {
        self.url = url.as_ref().clone();
        self
    }

    pub fn work(&self) -> &Work {
        &self.work
    }

    pub fn set_work(&mut self, work: Work) -> &mut Self {
        self.work = work;
        self
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn into_parse_task(self, html: &str) -> ParseTask {
        ParseTask {
            id: self.id,
            url: self.url,
            html: html.to_owned(),
            work: self.work,
        }
    }
}

impl<'de> FromDuktape<'de> for Task {
    fn from_context(ctx: &'de Context, index: Idx) -> DukResult<Self> {
        let t = ctx.get::<duktape::types::Object>(index)?;

        let url = Url::parse(t.get::<_, &str>("url")?).or_else::<DukError, _>(|e| {
            Err(DukErrorKind::TypeError("should be a url".to_owned()).into())
        })?;

        let work_p = t.get::<_, duktape::types::Ref>("work")?;

        let work = match work_p.get_type() {
            Type::String => Work::Path(None, work_p.get::<String>()?),
            _ => return Err(DukErrorKind::TypeError(format!("invalid error")).into()),
        };

        let id = if t.has("id") && t.get::<_, Ref>("id").unwrap().is(Type::String) {
            match t.get::<_, String>("id").unwrap().parse::<Uuid>() {
                Ok(id) => id,
                Err(_) => Uuid::new_v4(),
            }
        } else {
            Uuid::new_v4()
        };

        Ok(Task { id, url, work })
    }
}

impl ToDuktape for Task {
    fn to_context(self, ctx: &Context) -> DukResult<()> {
        let o = ctx.create::<duktape::types::Object>()?;

        let w = ctx.create::<duktape::types::Object>()?;
        match self.work {
            Work::Path(r, p) => {
                let p = match r {
                    None => p,
                    Some(r) => r.join(p).into_os_string().into_string().unwrap(),
                };
                w.set("type", "path");
                w.set("value", p);
            }
        };

        o.set("url", self.url.as_str()).set("work", w);
        o.set("id", self.id.to_hyphenated().to_string());

        ctx.push(o)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct ParseTask {
    pub(crate) id: Uuid,
    pub(crate) url: Url,
    pub(crate) html: String,
    pub(crate) work: Work,
}

impl<'de> FromDuktape<'de> for ParseTask {
    fn from_context(ctx: &'de Context, index: Idx) -> DukResult<Self> {
        let t = ctx.get::<duktape::types::Object>(index)?;

        let url = Url::parse(t.get::<_, &str>("url")?).or_else::<DukError, _>(|e| {
            Err(DukErrorKind::TypeError("should be a url".to_owned()).into())
        })?;

        let work_p = t.get::<_, duktape::types::Ref>("work")?;

        let work = match work_p.get_type() {
            Type::String => Work::Path(None, work_p.get::<String>()?),
            _ => return Err(DukErrorKind::TypeError(format!("invalid error")).into()),
        };

        let id = if t.has("id") && t.get::<_, Ref>("id").unwrap().is(Type::String) {
            match Uuid::parse_str(t.get::<_, &str>("id").unwrap()) {
                Ok(id) => id,
                Err(_) => Uuid::new_v4(),
            }
        } else {
            Uuid::new_v4()
        };

        let html = t.get::<_, String>("html").unwrap_or("".to_string());

        Ok(ParseTask {
            id,
            url,
            work,
            html,
        })
    }
}

impl ToDuktape for ParseTask {
    fn to_context(self, ctx: &Context) -> DukResult<()> {
        let o = ctx.create::<duktape::types::Object>()?;

        let w = ctx.create::<duktape::types::Object>()?;
        match self.work {
            Work::Path(r, p) => {
                let p = match r {
                    None => p,
                    Some(r) => r.join(p).into_os_string().into_string().unwrap(),
                };
                w.set("type", "path");
                w.set("value", p);
            }
        };

        o.set("url", self.url.as_str()).set("work", w);
        o.set("id", self.id.to_hyphenated().to_string());
        o.set("html", self.html);

        ctx.push(o)?;

        Ok(())
    }
}

impl ToDuktape for &ParseTask {
    fn to_context(self, ctx: &Context) -> DukResult<()> {
        let o = ctx.create::<duktape::types::Object>()?;

        let w = ctx.create::<duktape::types::Object>()?;
        match &self.work {
            Work::Path(r, p) => {
                let p = match r {
                    None => p.clone(),
                    Some(r) => r.join(p).into_os_string().into_string().unwrap(),
                };
                w.set("type", "path");
                w.set("value", p);
            }
        };

        o.set("url", self.url.as_str()).set("work", w);
        o.set("id", self.id.to_hyphenated().to_string());
        o.set("html", &self.html);

        ctx.push(o)?;

        Ok(())
    }
}
