use duktape2::prelude::{Context as DukContext,Require, file_resolver, Compile, Environment, ContextCommonJS, FromDuktapeContext,JSFunction, Function, JSValue, JSObject, Object};
use super::super::super::context::{Context};
use super::super::super::error::CrawlResult;
use vfs::physical::PhysicalFS;
use conveyor_work::{package::Package};
use super::super::super::work::WorkOutput;
use conveyor::Result;

pub(crate) static REQUIRE_JS: &'static str = include_str!("./runtime.js");


pub struct VM {
    inner: DukContext,
    ctx: Context,
    pub(crate) script: String,
}

impl VM {
    pub fn new<S: AsRef<str>>(mut ctx: Context, path:S) -> VM {

        let duk = DukContext::new().unwrap();

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

    pub fn ctx(&self) -> &Context {
        &self.ctx
    }

    pub fn run(&self, package:(String, Vec<u8>)) -> Result<Vec<WorkOutput<Package>>> {

        let module = self.inner.require(&self.script).unwrap().push();
        let function: Function = self.inner.getp().unwrap();
        let p = self.inner.get_global_string("Package").getp::<Function>().unwrap();
        // p.set("name", &package.0).unwrap();
        // p.set("content", &package.1).unwrap();

        let p = p.construct::<_, Object>((&package.0, &package.1)).unwrap();

        function.call::<_, Object>(p).unwrap().push();

        println!("ctx {:?}",self.inner);

        //println!("module: {}", module);

        return Ok(vec![WorkOutput::Result(Ok(Package::new(package.0, package.1)))])
    }
}