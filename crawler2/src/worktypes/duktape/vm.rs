use duktape2::prelude::*;
use super::super::super::context::{Context as CrawlContext};
use super::super::super::error::CrawlResult;
use vfs::physical::PhysicalFS;
use conveyor_work::{package::Package};
use super::super::super::work::WorkOutput;
use conveyor::{Result, ConveyorError};

pub(crate) static REQUIRE_JS: &'static str = include_str!("./runtime.js");


pub struct VM {
    inner: Context,
    ctx: CrawlContext,
    pub(crate) script: String,
}

impl VM {
    pub fn new<S: AsRef<str>>(mut ctx: CrawlContext, path:S) -> VM {

        let duk = Context::new().unwrap();

        Require::new()
        .env(Environment::from_env().unwrap())
        .resolver(
            "file",
            file_resolver(
                PhysicalFS::new("/").unwrap(),
                ctx.root().target().env().cwd().to_str().expect("invalid path"),
            ),
        )
        .build(&duk)
        .expect("require");


        {
            let requirejs: Function = duk
            .push_string(REQUIRE_JS)
            .push_string("runtime.js")
            .compile(Compile::EVAL).unwrap()
            .call(0).unwrap()
            .getp().unwrap();

            requirejs.call::<_, Function>(duk.push_global_object().getp::<Object>().unwrap()).unwrap();
        }



        duk.require(path.as_ref()).expect("invalid script");

        VM{
            inner:duk,
            ctx,
            script: path.as_ref().to_string(),
        }
    }

    pub fn ctx(&self) -> &CrawlContext {
        &self.ctx
    }

    pub fn run(&self, package:(String, Vec<u8>)) -> Result<Vec<WorkOutput<Package>>> {

        let module = self.inner.require(&self.script).unwrap().push();
        let function: Function = self.inner.getp().unwrap();
        let p_ctor = self.inner.get_global_string("Package").getp::<Function>().unwrap();
        let p = p_ctor.construct::<_, Object>((&package.0, package.1.as_slice())).unwrap();
        let re = function.call::<_, Array>(p).unwrap();
        let pack = parse(&re).unwrap();

        // let pack = match deserialize_reference(&re) {
        //     Err(e) => unimplemented!("could not do"),
        //     Ok(s) => match s {
        //         ValueOrBytes::Bytes(bs) => Package::new(package.0, bs),
        //         ValueOrBytes::Value(v)  => {
        //             println!("{}", serde_json::to_string_pretty(&v).unwrap());
        //             Package::new(package.0, v)
        //         },
        //     }
        // };

        Ok(pack)

        // return Ok(vec![WorkOutput::Result(Ok(pack))])
    }
}


fn parse(array: &Array) -> DukResult<Vec<WorkOutput<Package>>> {
    let iter = array.iter();
    for entry in iter {
        let o: Object = entry.to()?;
        let w = match o.get::<_, &str>("type")? {
            "ok" => {
                WorkOutput::Result(Ok(parse_package(&o.get::<_,Object>("package")?)?))
            },
            "then" => {
                WorkOutput::Then(parse_package(&o.get::<_,Object>("package")?)?)
            },
            "err" => {
                unimplemented!("Error {}", o)
            }
            _ => {
                 unimplemented!("Error")
            }
        };
    }

    Ok(Vec::new())
}

fn parse_package(package: &Object) -> DukResult<Package> {
    let name: &str = package.get("name")?;
    let content: Reference = package.get("content")?;

    let pack = match content.get_type() {
        Type::String => Package::new(name, content.to::<&str>()?),
        Type::Buffer => Package::new(name, content.to::<&[u8]>()?),
        _ => return Err(DukError::new(DukErrorCode::Type, format!("invalid package content type: {:?}", content.get_type()))),
    };

    Ok(pack)
}

use serde_json::{Value,Number, Map};

#[derive(Debug)]
enum ValueOrBytes {
    Value(Value),
    Bytes(Vec<u8>),
}

fn decode(ctx: &Context, idx:Idx) -> DukResult<ValueOrBytes> {
    let re: Reference = ctx.get(idx)?;
    deserialize_reference(&re)
}

fn deserialize_reference2(value: &Reference) -> DukResult<Value> {
    let val = match value.get_type() {
        Type::Array => {
            let re = value.to()?;
            deserialize_array(&re)?
        },
        Type::String => Value::String(value.to()?),
        Type::Null | Type::Undefined => Value::Null,
        Type::Boolean => Value::Bool(value.to()?),
        Type::Number => Value::Number(Number::from_f64(value.to()?).unwrap()),
        Type::Object => {
            let o:Object = value.to()?;
            let iter = o.iter();
            let mut out = Map::new();
            for e in iter {
                out.insert(e.0.to_string(), deserialize_reference2(&e.1)?);
            }
            Value::Object(out)
        }
        _ => {
            unimplemented!("reference {:?}", value.get_type());
        }
    };

    Ok(val)
}

fn deserialize_reference(value: &Reference) -> DukResult<ValueOrBytes> {
    let val = match value.get_type() {
        Type::Array => {
            let re = value.to()?;
            deserialize_array(&re)?
        },
        Type::String => Value::String(value.to()?),
        Type::Null | Type::Undefined => Value::Null,
        Type::Boolean => Value::Bool(value.to()?),
        Type::Number => Value::Number(Number::from_f64(value.to()?).unwrap()),
        Type::Buffer => {
            let bs = value.to()?;
            return Ok(ValueOrBytes::Bytes(bs));
        }
        Type::Object => {
            let o:Object = value.to()?;
            let iter = o.iter();
            let mut out = Map::new();
            for e in iter {
                out.insert(e.0.to_string(), deserialize_reference2(&e.1)?);
            }
            Value::Object(out)
        }
        _ => {
            unimplemented!("reference {:?}", value.get_type());
        }
    };

    Ok(ValueOrBytes::Value(val))
}

fn deserialize_array(array: &Array) -> DukResult<Value> {

    let mut out = Vec::with_capacity(array.len());
    
    for item in array.iter() {
        out.push(match deserialize_reference(&item)? {
            ValueOrBytes::Value(v) => v,
            ValueOrBytes::Bytes(b) => return Err(DukError::new(DukErrorCode::Type, "could not be bytes"))
        });
    }

    Ok(Value::Array(out))

}