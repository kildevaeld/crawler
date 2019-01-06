use super::cheerio::CHEERIO_SOURCE;
use super::error::Result;
use super::queue::TaskQueue;
use super::task::{ParseTask, Task, Work};
use super::utils;
use super::utils::measure;
use crossbeam_channel::Sender;
use duktape::error::{ErrorKind, Result as DukResult};
use duktape::prelude::*;
use duktape_modules::{self, require, CJSContext};

struct SenderKey;

impl duktape::Key for SenderKey {
    type Value = Sender<Task>;
}

pub struct VM {
    ctx: Context,
}

impl VM {
    pub fn new(sender: Sender<Task>) -> Result<VM> {
        let ctx = Context::new()?;

        let mut builder = duktape_modules::Builder::new();

        builder.module("cheerio", |ctx: &Context| {
            let module: Object = ctx.get(-1)?;
            require::eval_module(ctx, CHEERIO_SOURCE, &module).unwrap();
            Ok(1)
        });

        duktape_modules::register(&ctx, builder)?;

        ctx.data_mut()?.insert::<SenderKey>(sender);

        let mut class = duktape::class::build();

        class.method(
            "push",
            (1, |ctx: &Context, _this: &mut class::Instance| {
                let mut task = ctx.get::<Task>(0)?;

                let root = ctx
                    .push_this()
                    .get_prop_string(-1, "parentTask")
                    .get_prop_string(-1, "root")
                    .get::<&str>(-1)?;

                if task.root() == "" {
                    task.set_root(root.to_string());
                }

                let sender = ctx.data()?.get::<SenderKey>().unwrap();
                sender.send(task);
                ctx.push_current_function();
                Ok(1)
            }),
        );

        ctx.push_global_object()
            .getp::<Object>()?
            .set("Queue", class);

        VM::build_context(&ctx)?;

        ctx.get_global_string("require")
            .getp::<Function>()?
            .call::<_, Object>("cheerio")?;

        Ok(VM { ctx })
    }

    fn build_context(ctx: &Context) -> Result<()> {
        let mut class = duktape::class::build();

        class
            .constructor((1, |ctx: &Context, _this: &mut class::Instance| {
                //let obj = ctx.push_this().getp::<Object>()?;
                let task = ctx.get::<Ref>(0)?;
                ctx.push_this().getp::<Object>()?.set("task", task);

                Ok(0)
            }))
            .method(
                "log",
                (1, |ctx: &Context, this: &mut class::Instance| {
                    ctx.push_current_function();
                    Ok(1)
                }),
            );

        ctx.push_global_object()
            .push(class)?
            .put_prop_string(-2, "Context");

        Ok(())
    }

    pub fn run<'a>(&'a self, task: &ParseTask) -> Result<Ref<'a>> {
        let queue = self
            .ctx
            .get_global_string("Queue")
            .construct(0)?
            .getp::<Object>()?;

        queue.set("parentTask", task);

        let module = match &task.work {
            Work::Path(p) => {
                let path = utils::join(&task.root, p)?;
                let (elapsed, module) = measure(|| self.ctx.eval_main(&path));
                let module = module?;
                info!("script {:?} evaluated in {:?}", path, elapsed);
                module
            }
        };

        let exports = module.get::<_, Function>("exports")?;

        let cheerio = self
            .ctx
            .get_global_string("require")
            .getp::<Function>()?
            .call::<_, Object>("cheerio")?;

        let (elapsed, instance) =
            measure(|| cheerio.call::<_, _, Object>("load", task.html.as_str()));
        let instance = instance?;
        info!("dom loaded in {:?}", elapsed);

        let context = self
            .ctx
            .get_global_string("Context")
            .push(task)?
            .construct(1)?
            .getp::<Object>()?;

        let (elapsed, out) = measure(|| exports.call::<_, Ref>((instance, queue, context)));
        let out = out?;
        info!("page processed in {:?}", elapsed);

        Ok(out)
    }

    pub fn run2<'a>(&'a self, task: &Task, html: &str) -> Result<Ref<'a>> {
        let queue = self
            .ctx
            .get_global_string("Queue")
            .construct(0)?
            .getp::<Object>()?;

        queue.set("parentTask", task);

        let module = match task.work() {
            Work::Path(p) => {
                let path = utils::join(&task.root(), p)?;
                let (elapsed, module) = measure(|| self.ctx.eval_main(&path));
                let module = module?;
                info!("script {:?} evaluated in {:?}", path, elapsed);
                module
            }
        };

        let exports = module.get::<_, Function>("exports")?;

        let cheerio = self
            .ctx
            .get_global_string("require")
            .getp::<Function>()?
            .call::<_, Object>("cheerio")?;

        let (elapsed, instance) = measure(|| cheerio.call::<_, _, Object>("load", html));
        let instance = instance?;
        info!("dom loaded in {:?}", elapsed);

        let context = self
            .ctx
            .get_global_string("Context")
            .push(task)?
            .construct(1)?
            .getp::<Object>()?;

        let (elapsed, out) = measure(|| exports.call::<_, Ref>((instance, queue, context)));
        let out = out?;
        info!("page processed in {:?}", elapsed);

        Ok(out)
    }
}
