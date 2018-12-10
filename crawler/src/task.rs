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
    Path(String),
}

impl Work {
    pub fn path<T: AsRef<str>>(path: T) -> Work {
        // let root = match root {
        //     Some(root) => Some(root.as_ref().to_path_buf()),
        //     None => None,
        // };
        Work::Path(path.as_ref().to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Task {
    id: Uuid,
    #[serde(with = "url_serde")]
    url: Url,
    work: Work,
    root: String,
}

impl Task {
    pub fn new<T: AsRef<str>>(root: T, url: T, work: Work) -> Result<Task> {
        let url = Url::parse(url.as_ref())?;
        let id = Uuid::new_v4();
        Ok(Task {
            work,
            url,
            id,
            root: root.as_ref().to_string(),
        })
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

    pub fn set_root(&mut self, root: String) -> &mut Self {
        self.root = root;
        self
    }

    pub fn root(&self) -> &str {
        self.root.as_str()
    }

    pub fn into_parse_task(self, html: &str) -> ParseTask {
        ParseTask {
            id: self.id,
            url: self.url,
            html: html.to_owned(),
            work: self.work,
            root: self.root,
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
            Type::String => Work::Path(work_p.get::<String>()?),
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

        let root = if t.has("root") && t.get::<_, Ref>("root").unwrap().is(Type::String) {
            t.get::<_, String>("root")?
        } else {
            "".to_string()
        };

        Ok(Task {
            id,
            url,
            work,
            root,
        })
    }
}

impl ToDuktape for Task {
    fn to_context(self, ctx: &Context) -> DukResult<()> {
        let o = ctx.create::<duktape::types::Object>()?;

        let w = ctx.create::<duktape::types::Object>()?;
        match self.work {
            Work::Path(p) => {
                w.set("type", "path");
                w.set("value", p);
            }
        };

        o.set("url", self.url.as_str()).set("work", w);
        o.set("id", self.id.to_hyphenated().to_string());
        o.set("root", self.root);

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
    pub(crate) root: String,
}

impl<'de> FromDuktape<'de> for ParseTask {
    fn from_context(ctx: &'de Context, index: Idx) -> DukResult<Self> {
        let t = ctx.get::<duktape::types::Object>(index)?;

        let url = Url::parse(t.get::<_, &str>("url")?).or_else::<DukError, _>(|e| {
            Err(DukErrorKind::TypeError("should be a url".to_owned()).into())
        })?;

        let work_p = t.get::<_, duktape::types::Ref>("work")?;

        let work = match work_p.get_type() {
            Type::String => Work::Path(work_p.get::<String>()?),
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

        let root = t.get::<_, String>("root")?;

        Ok(ParseTask {
            id,
            url,
            work,
            html,
            root,
        })
    }
}

impl ToDuktape for ParseTask {
    fn to_context(self, ctx: &Context) -> DukResult<()> {
        let o = ctx.create::<duktape::types::Object>()?;

        let w = ctx.create::<duktape::types::Object>()?;
        match self.work {
            Work::Path(p) => {
                w.set("type", "path");
                w.set("value", p);
            }
        };

        o.set("url", self.url.as_str()).set("work", w);
        o.set("id", self.id.to_hyphenated().to_string());
        o.set("html", self.html);
        o.set("root", self.root);

        ctx.push(o)?;

        Ok(())
    }
}

impl ToDuktape for &ParseTask {
    fn to_context(self, ctx: &Context) -> DukResult<()> {
        let o = ctx.create::<duktape::types::Object>()?;

        let w = ctx.create::<duktape::types::Object>()?;
        match &self.work {
            Work::Path(p) => {
                w.set("type", "path");
                w.set("value", p);
            }
        };

        o.set("url", self.url.as_str()).set("work", w);
        o.set("id", self.id.to_hyphenated().to_string());
        o.set("html", &self.html);
        o.set("root", &self.root);

        ctx.push(o)?;

        Ok(())
    }
}
